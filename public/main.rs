// ╔══════════════════════════════════════════════════════════════════╗
// ║         mc-tunnel v0.3 — Minecraft reverse-proxy tunnel          ║
// ║                    + Web Dashboard API                            ║
// ║                                                                   ║
// ║  Маркеры:                                                         ║
// ║    //vps    — код только для VPS                                  ║
// ║    //client — код только для клиента (дом)                        ║
// ║    //web    — код для веб-дашборда                                ║
// ║    без маркера — общий код                                        ║
// ╚══════════════════════════════════════════════════════════════════╝

use axum::{
    routing::get,
    Router,
    Json,
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use tracing::{error, info, warn};

// ── Протокол хэндшейка ──────────────────────────────────────────────
const HELO: &[u8; 4] = b"HELO";          // control: client → vps
const OKOK: &[u8; 4] = b"OKOK";          // control: vps → client
const SIG_NEW_PLAYER: u8 = 0x01;         // control: vps → client "новый игрок"
const DATA_HANDSHAKE: u8 = 0x02;         // data: client → vps "я data-канал"
const DATA_ACK: u8 = 0x03;               // data: vps → client "принял"

// ═════════════════════════════════════════════════════════════════════
//web ── СТАТИСТИКА ДЛЯ WEB DASHBOARD ─────────────────────────────────
// ═════════════════════════════════════════════════════════════════════

#[derive(Debug)]                                                        //web
pub struct AppStats {                                                   //web
    /// Количество активных соединений                                  //web
    pub active_connections: AtomicUsize,                                //web
    /// Общий трафик в байтах                                           //web
    pub total_bytes_transferred: AtomicU64,                             //web
    /// Статус туннеля (true = connected)                               //web
    pub tunnel_connected: RwLock<bool>,                                 //web
    /// Время запуска для расчёта uptime                                //web
    pub start_time: Instant,                                            //web
    /// Пиковое количество соединений                                   //web
    pub peak_connections: AtomicUsize,                                  //web
    /// Общее количество соединений за всё время                        //web
    pub total_connections: AtomicU64,                                   //web
}                                                                       //web

impl AppStats {                                                         //web
    pub fn new() -> Self {                                              //web
        Self {                                                          //web
            active_connections: AtomicUsize::new(0),                    //web
            total_bytes_transferred: AtomicU64::new(0),                 //web
            tunnel_connected: RwLock::new(false),                       //web
            start_time: Instant::now(),                                 //web
            peak_connections: AtomicUsize::new(0),                      //web
            total_connections: AtomicU64::new(0),                       //web
        }                                                               //web
    }                                                                   //web
                                                                        //web
    /// Увеличить счётчик активных соединений                           //web
    pub fn connection_opened(&self) {                                   //web
        let current = self.active_connections.fetch_add(1, Ordering::SeqCst) + 1; //web
        self.total_connections.fetch_add(1, Ordering::SeqCst);          //web
        // Обновляем пик если нужно                                     //web
        let mut peak = self.peak_connections.load(Ordering::SeqCst);    //web
        while current > peak {                                          //web
            match self.peak_connections.compare_exchange_weak(          //web
                peak, current, Ordering::SeqCst, Ordering::SeqCst       //web
            ) {                                                         //web
                Ok(_) => break,                                         //web
                Err(p) => peak = p,                                     //web
            }                                                           //web
        }                                                               //web
    }                                                                   //web
                                                                        //web
    /// Уменьшить счётчик активных соединений                           //web
    pub fn connection_closed(&self) {                                   //web
        self.active_connections.fetch_sub(1, Ordering::SeqCst);         //web
    }                                                                   //web
                                                                        //web
    /// Добавить переданные байты                                       //web
    pub fn add_bytes(&self, bytes: u64) {                               //web
        self.total_bytes_transferred.fetch_add(bytes, Ordering::SeqCst);//web
    }                                                                   //web
                                                                        //web
    /// Установить статус туннеля                                       //web
    pub async fn set_tunnel_status(&self, connected: bool) {            //web
        let mut status = self.tunnel_connected.write().await;           //web
        *status = connected;                                            //web
    }                                                                   //web
}                                                                       //web

/// JSON-ответ для /api/stats                                           //web
#[derive(Serialize)]                                                    //web
pub struct StatsResponse {                                              //web
    pub active_connections: usize,                                      //web
    pub total_bytes_transferred: u64,                                   //web
    pub tunnel_status: String,                                          //web
    pub uptime_secs: u64,                                               //web
    pub peak_connections: usize,                                        //web
    pub total_connections: u64,                                         //web
}                                                                       //web

/// Обработчик GET /api/stats                                           //web
async fn get_stats(                                                     //web
    axum::extract::State(stats): axum::extract::State<Arc<AppStats>>    //web
) -> Json<StatsResponse> {                                              //web
    let tunnel_connected = *stats.tunnel_connected.read().await;        //web
    Json(StatsResponse {                                                //web
        active_connections: stats.active_connections.load(Ordering::SeqCst), //web
        total_bytes_transferred: stats.total_bytes_transferred.load(Ordering::SeqCst), //web
        tunnel_status: if tunnel_connected { "connected".into() } else { "disconnected".into() }, //web
        uptime_secs: stats.start_time.elapsed().as_secs(),              //web
        peak_connections: stats.peak_connections.load(Ordering::SeqCst),//web
        total_connections: stats.total_connections.load(Ordering::SeqCst), //web
    })                                                                  //web
}                                                                       //web

/// Запуск веб-сервера на отдельном порту                               //web
async fn start_web_server(stats: Arc<AppStats>, port: u16) {            //web
    let cors = CorsLayer::new()                                         //web
        .allow_origin(Any)                                              //web
        .allow_methods([Method::GET])                                   //web
        .allow_headers(Any);                                            //web
                                                                        //web
    let app = Router::new()                                             //web
        .route("/api/stats", get(get_stats))                            //web
        .layer(cors)                                                    //web
        .with_state(stats);                                             //web
                                                                        //web
    let addr = format!("0.0.0.0:{}", port);                             //web
    info!("[web] 🌐 Dashboard API → http://{}", addr);                  //web
                                                                        //web
    let listener = tokio::net::TcpListener::bind(&addr).await           //web
        .unwrap_or_else(|e| panic!("[web] bind {addr}: {e}"));          //web
                                                                        //web
    axum::serve(listener, app).await                                    //web
        .unwrap_or_else(|e| error!("[web] server error: {e}"));         //web
}                                                                       //web

// ── Config (.toml) ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Config {
    mode: String,
    #[serde(default = "default_game_port")]
    game_port: u16,
    #[serde(default = "default_tunnel_port")]
    tunnel_port: u16,
    #[serde(default = "default_vps_address")]
    vps_address: String,
    #[serde(default = "default_mc_address")]
    minecraft_address: String,
    #[serde(default = "default_reconnect_delay")]
    reconnect_delay_secs: u64,
    #[serde(default)]
    max_reconnect_attempts: u32,
    #[serde(default = "default_web_port")]                              //web
    web_port: u16,                                                      //web
}

fn default_game_port() -> u16 { 25565 }
fn default_tunnel_port() -> u16 { 9000 }
fn default_vps_address() -> String { "127.0.0.1:9000".into() }
fn default_mc_address() -> String { "127.0.0.1:25565".into() }
fn default_reconnect_delay() -> u64 { 3 }
fn default_web_port() -> u16 { 3001 }                                   //web

impl Config {
    fn load() -> Self {
        let path = std::env::args().nth(1).unwrap_or("config.toml".into());
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Не удалось прочитать {path}: {e}"));
        toml::from_str(&text)
            .unwrap_or_else(|e| panic!("Ошибка парсинга {path}: {e}"))
    }
}

// ── Утилита: двусторонний мост с подсчётом трафика ──────────────────

async fn bridge(mut a: TcpStream, mut b: TcpStream, label: &str, stats: Arc<AppStats>) {
    // Регистрируем открытие соединения                                 //web
    stats.connection_opened();                                          //web
    
    match io::copy_bidirectional(&mut a, &mut b).await {
        Ok((up, down)) => {
            info!("[{label}] закрыт — ↑{up} ↓{down} байт");
            stats.add_bytes(up + down);                                 //web
        }
        Err(e) => warn!("[{label}] разрыв — {e}"),
    }
    
    // Регистрируем закрытие соединения                                 //web
    stats.connection_closed();                                          //web
}

// ═════════════════════════════════════════════════════════════════════
//vps ── СЕРВЕРНАЯ ЧАСТЬ (VPS) ────────────────────────────────────────
// ═════════════════════════════════════════════════════════════════════

//vps Ожидаем control-подключение от клиента, проверяем хэндшейк
//vps Возвращаем control_socket
async fn vps_wait_for_client(tunnel_ln: &TcpListener, stats: &Arc<AppStats>) -> Option<TcpStream> { //vps
    loop {                                                              //vps
        let (mut sock, addr) = match tunnel_ln.accept().await {         //vps
            Ok(v) => v,                                                 //vps
            Err(e) => {                                                 //vps
                error!("[vps] accept ошибка: {e}");                     //vps
                continue;                                               //vps
            }                                                           //vps
        };                                                              //vps
        // Читаем 4 байта хэндшейка                                     //vps
        let mut buf = [0u8; 4];                                         //vps
        if sock.read_exact(&mut buf).await.is_err() {                   //vps
            warn!("[vps] {addr} — не прислал хэндшейк, сброс");         //vps
            continue;                                                   //vps
        }                                                               //vps
        if &buf == HELO {                                               //vps
            info!("[vps] ✔ клиент {addr} подключился (control)");       //vps
            if sock.write_all(OKOK).await.is_err() {                    //vps
                warn!("[vps] не удалось отправить OKOK");               //vps
                continue;                                               //vps
            }                                                           //vps
            stats.set_tunnel_status(true).await;                        //vps //web
            return Some(sock);                                          //vps
        }                                                               //vps
        warn!("[vps] {addr} — неизвестный хэндшейк {:?}, сброс", buf);  //vps
    }                                                                   //vps
}                                                                       //vps

//vps Ожидаем data-подключение от клиента с хэндшейком 0x02
async fn vps_accept_data(tunnel_ln: &TcpListener) -> Option<TcpStream> { //vps
    // Таймаут 10 сек — если клиент не прислал data-канал, сбрасываем   //vps
    let deadline = sleep(Duration::from_secs(10));                      //vps
    tokio::pin!(deadline);                                              //vps
    loop {                                                              //vps
        tokio::select! {                                                //vps
            _ = &mut deadline => {                                      //vps
                warn!("[vps] таймаут ожидания data-канала");            //vps
                return None;                                            //vps
            }                                                           //vps
            result = tunnel_ln.accept() => {                            //vps
                let (mut sock, addr) = match result {                   //vps
                    Ok(v) => v,                                         //vps
                    Err(e) => {                                         //vps
                        error!("[vps] data accept ошибка: {e}");        //vps
                        continue;                                       //vps
                    }                                                   //vps
                };                                                      //vps
                let mut byte = [0u8; 1];                                //vps
                if sock.read_exact(&mut byte).await.is_err() {          //vps
                    warn!("[vps] {addr} — data: не прислал хэндшейк");  //vps
                    continue;                                           //vps
                }                                                       //vps
                if byte[0] == DATA_HANDSHAKE {                          //vps
                    info!("[vps] ✔ data-канал от {addr}");              //vps
                    let _ = sock.write_all(&[DATA_ACK]).await;          //vps
                    return Some(sock);                                  //vps
                }                                                       //vps
                warn!("[vps] {addr} — data: неверный байт 0x{:02X}", byte[0]); //vps
            }                                                           //vps
        }                                                               //vps
    }                                                                   //vps
}                                                                       //vps

//vps Главный цикл VPS
async fn vps_main(cfg: Arc<Config>, stats: Arc<AppStats>) {             //vps
    let game_addr = format!("0.0.0.0:{}", cfg.game_port);               //vps
    let tunnel_addr = format!("0.0.0.0:{}", cfg.tunnel_port);           //vps
    let game_ln = TcpListener::bind(&game_addr).await                   //vps
        .unwrap_or_else(|e| panic!("[vps] bind {game_addr}: {e}"));     //vps
    let tunnel_ln = TcpListener::bind(&tunnel_addr).await               //vps
        .unwrap_or_else(|e| panic!("[vps] bind {tunnel_addr}: {e}"));   //vps
    info!("[vps] 🎮 игроки    → {game_addr}");                          //vps
    info!("[vps] 🔗 туннель   → {tunnel_addr}");                        //vps
    // Внешний цикл: ждём подключения клиента                           //vps
    loop {                                                              //vps
        info!("[vps] ожидание клиента...");                             //vps
        stats.set_tunnel_status(false).await;                           //vps //web
        let control = match vps_wait_for_client(&tunnel_ln, &stats).await { //vps
            Some(c) => c,                                               //vps
            None => continue,                                           //vps
        };                                                              //vps
        // Arc для того чтобы контрол шарить между тасками (write half) //vps
        let control = Arc::new(tokio::sync::Mutex::new(control));       //vps
        info!("[vps] клиент подключён, слушаем игроков…");              //vps
        // Внутренний цикл: обрабатываем игроков пока control жив       //vps
        loop {                                                          //vps
            let (player_sock, player_addr) = match game_ln.accept().await { //vps
                Ok(v) => v,                                             //vps
                Err(e) => {                                             //vps
                    error!("[vps] game accept: {e}");                   //vps
                    continue;                                           //vps
                }                                                       //vps
            };                                                          //vps
            info!("[vps] 🎮 игрок {player_addr}");                      //vps
            // Отправляем сигнал клиенту                                //vps
            {                                                           //vps
                let mut ctrl = control.lock().await;                    //vps
                if ctrl.write_all(&[SIG_NEW_PLAYER]).await.is_err() {   //vps
                    warn!("[vps] control разорван, ждём переподключения"); //vps
                    stats.set_tunnel_status(false).await;               //vps //web
                    break; // → внешний цикл                            //vps
                }                                                       //vps
            }                                                           //vps
            // Ждём data-канал                                          //vps
            let data_sock = match vps_accept_data(&tunnel_ln).await {   //vps
                Some(s) => s,                                           //vps
                None => {                                               //vps
                    warn!("[vps] не получили data-канал, дропаем игрока"); //vps
                    continue;                                           //vps
                }                                                       //vps
            };                                                          //vps
            // Спариваем                                                //vps
            let label = format!("player:{player_addr}");                //vps
            let stats_clone = stats.clone();                            //vps //web
            tokio::spawn(async move {                                   //vps
                bridge(player_sock, data_sock, &label, stats_clone).await; //vps
            });                                                         //vps
        }                                                               //vps
    }                                                                   //vps
}                                                                       //vps

// ═════════════════════════════════════════════════════════════════════
//client ── КЛИЕНТСКАЯ ЧАСТЬ (ДОМ) ────────────────────────────────────
// ═════════════════════════════════════════════════════════════════════

//client Подключение control-канала с хэндшейком
async fn client_connect_control(addr: &str) -> io::Result<TcpStream> {  //client
    let mut sock = TcpStream::connect(addr).await?;                     //client
    sock.write_all(HELO).await?;                                        //client
    let mut buf = [0u8; 4];                                             //client
    sock.read_exact(&mut buf).await?;                                   //client
    if &buf != OKOK {                                                   //client
        return Err(io::Error::new(                                      //client
            io::ErrorKind::InvalidData,                                 //client
            format!("VPS ответил {:?} вместо OKOK", buf),               //client
        ));                                                             //client
    }                                                                   //client
    Ok(sock)                                                            //client
}                                                                       //client

//client Открытие data-канала с хэндшейком 0x02
async fn client_open_data(addr: &str) -> io::Result<TcpStream> {        //client
    let mut sock = TcpStream::connect(addr).await?;                     //client
    sock.write_all(&[DATA_HANDSHAKE]).await?;                           //client
    let mut ack = [0u8; 1];                                             //client
    sock.read_exact(&mut ack).await?;                                   //client
    if ack[0] != DATA_ACK {                                             //client
        return Err(io::Error::new(                                      //client
            io::ErrorKind::InvalidData,                                 //client
            format!("VPS data ack = 0x{:02X}", ack[0]),                  //client
        ));                                                             //client
    }                                                                   //client
    Ok(sock)                                                            //client
}                                                                       //client

//client Главный цикл клиента с reconnect-логикой
async fn client_main(cfg: Arc<Config>, stats: Arc<AppStats>) {          //client
    let vps_addr: Arc<str> = Arc::from(cfg.vps_address.as_str());       //client
    let mc_addr: Arc<str> = Arc::from(cfg.minecraft_address.as_str());  //client
    let delay = Duration::from_secs(cfg.reconnect_delay_secs);          //client
    let max_attempts = cfg.max_reconnect_attempts;                      //client
    let mut attempt: u32 = 0;                                           //client
    // Reconnect loop                                                   //client
    loop {                                                              //client
        attempt += 1;                                                   //client
        if max_attempts > 0 && attempt > max_attempts {                 //client
            error!("[client] исчерпаны попытки ({max_attempts}), выход"); //client
            break;                                                      //client
        }                                                               //client
        info!("[client] подключение к VPS {vps_addr} (попытка {attempt})…"); //client
        let mut control = match client_connect_control(&vps_addr).await { //client
            Ok(c) => {                                                  //client
                info!("[client] ✔ control установлен");                 //client
                stats.set_tunnel_status(true).await;                    //client //web
                attempt = 0; // сброс счётчика при успехе               //client
                c                                                       //client
            }                                                           //client
            Err(e) => {                                                 //client
                warn!("[client] ✘ не удалось: {e}");                    //client
                stats.set_tunnel_status(false).await;                   //client //web
                sleep(delay).await;                                     //client
                continue;                                               //client
            }                                                           //client
        };                                                              //client
        // Слушаем сигналы                                              //client
        loop {                                                          //client
            let mut sig = [0u8; 1];                                     //client
            if control.read_exact(&mut sig).await.is_err() {            //client
                warn!("[client] control разорван, переподключение…");   //client
                stats.set_tunnel_status(false).await;                   //client //web
                break;                                                  //client
            }                                                           //client
            if sig[0] != SIG_NEW_PLAYER {                               //client
                warn!("[client] неизвестный сигнал 0x{:02X}", sig[0]);  //client
                continue;                                               //client
            }                                                           //client
            info!("[client] ← сигнал: новый игрок");                    //client
            let vps = vps_addr.clone();  // Arc::clone — дёшево         //client
            let mc = mc_addr.clone();                                   //client
            let stats_clone = stats.clone();                            //client //web
            tokio::spawn(async move {                                   //client
                // 1) Data-канал к VPS                                  //client
                let data_sock = match client_open_data(&vps).await {    //client
                    Ok(s) => s,                                         //client
                    Err(e) => {                                         //client
                        error!("[client] data-канал: {e}");             //client
                        return;                                         //client
                    }                                                   //client
                };                                                      //client
                // 2) Подключение к локальному MC                       //client
                let mc_sock = match TcpStream::connect(&*mc).await {    //client
                    Ok(s) => s,                                         //client
                    Err(e) => {                                         //client
                        error!("[client] minecraft {mc}: {e}");         //client
                        return;                                         //client
                    }                                                   //client
                };                                                      //client
                info!("[client] ↔ мост VPS ↔ MC");                      //client
                bridge(data_sock, mc_sock, "tunnel", stats_clone).await;//client
            });                                                         //client
        }                                                               //client
        warn!("[client] переподключение через {delay:?}…");             //client
        sleep(delay).await;                                             //client
    }                                                                   //client
}                                                                       //client

// ═════════════════════════════════════════════════════════════════════
// ── ТОЧКА ВХОДА ─────────────────────────────────────────────────────
// ═════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    // Инициализация tracing (уровень через RUST_LOG, по умолчанию info)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = Arc::new(Config::load());
    let stats = Arc::new(AppStats::new());                              //web

    info!("mc-tunnel v0.3 | режим: {}", cfg.mode);

    // Запускаем веб-сервер в отдельном таске                           //web
    let web_stats = stats.clone();                                      //web
    let web_port = cfg.web_port;                                        //web
    tokio::spawn(async move {                                           //web
        start_web_server(web_stats, web_port).await;                    //web
    });                                                                 //web

    match cfg.mode.as_str() {
        "vps" => vps_main(cfg, stats).await,                            //vps
        "client" => client_main(cfg, stats).await,                      //client
        other => error!("Неизвестный режим «{other}», укажите mode = \"vps\" | \"client\" в config.toml"),
    }
}

// ═════════════════════════════════════════════════════════════════════
// ── CARGO.TOML (обновить зависимости) ───────────────────────────────
// ═════════════════════════════════════════════════════════════════════
//
// [package]
// name = "mc-tunnel"
// version = "0.3.0"
// edition = "2021"
//
// [dependencies]
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1", features = ["derive"] }
// toml = "0.8"
// tracing = "0.1"
// tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter"] }
// axum = "0.7"
// tower-http = { version = "0.5", features = ["cors"] }
//
// ═════════════════════════════════════════════════════════════════════
// ── CONFIG.TOML (пример) ────────────────────────────────────────────
// ═════════════════════════════════════════════════════════════════════
//
// # Для VPS:
// mode = "vps"
// game_port = 25565
// tunnel_port = 9000
// web_port = 3001      # <- новый параметр для Dashboard API
//
// # Для Client:
// mode = "client"
// vps_address = "YOUR_VPS_IP:9000"
// minecraft_address = "127.0.0.1:25565"
// reconnect_delay_secs = 3
// max_reconnect_attempts = 0
// web_port = 3001      # <- Dashboard API и на клиенте тоже
//
