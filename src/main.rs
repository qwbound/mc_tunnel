// ╔══════════════════════════════════════════════════════════════════╗
// ║ mc-tunnel v0.6 — Safe error handling + custom config path       ║
// ╚══════════════════════════════════════════════════════════════════╝

use axum::{routing::get, Router, Json, http::Method};
use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, error, warn};

const HELO: &[u8; 4] = b"HELO";
const OKOK: &[u8; 4] = b"OKOK";
const SIG_NEW_PLAYER: u8 = 0x01;
const DATA_HANDSHAKE: u8 = 0x02;
const DATA_ACK: u8 = 0x03;

// ═══════════════════════════════════════════════════════════════════
// AppStats — статистика туннеля
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct AppStats {
    active_connections: AtomicUsize,
    total_bytes_transferred: AtomicU64,
    tunnel_connected: RwLock<bool>,
    start_time: Instant,
    peak_connections: AtomicUsize,
    total_connections: AtomicU64,
    active_ips: RwLock<Vec<String>>,
}

impl AppStats {
    pub fn new() -> Self {
        Self {
            active_connections: AtomicUsize::new(0),
            total_bytes_transferred: AtomicU64::new(0),
            tunnel_connected: RwLock::new(false),
            start_time: Instant::now(),
            peak_connections: AtomicUsize::new(0),
            total_connections: AtomicU64::new(0),
            active_ips: RwLock::new(Vec::new()),
        }
    }

    pub fn get_active_connections(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }

    pub fn get_total_bytes_transferred(&self) -> u64 {
        self.total_bytes_transferred.load(Ordering::SeqCst)
    }

    pub async fn get_tunnel_status(&self) -> bool {
        *self.tunnel_connected.read().await
    }

    pub fn get_start_time(&self) -> Instant {
        self.start_time
    }

    pub fn get_peak_connections(&self) -> usize {
        self.peak_connections.load(Ordering::SeqCst)
    }

    pub fn get_total_connections(&self) -> u64 {
        self.total_connections.load(Ordering::SeqCst)
    }

    pub async fn get_active_ips(&self) -> Vec<String> {
        self.active_ips.read().await.clone()
    }

    pub async fn connection_opened(&self, ip: String) {
        let current = self.active_connections.fetch_add(1, Ordering::SeqCst) + 1;
        self.total_connections.fetch_add(1, Ordering::SeqCst);
        
        // Обновляем пиковое значение
        let mut peak = self.peak_connections.load(Ordering::SeqCst);
        while current > peak {
            match self.peak_connections.compare_exchange_weak(
                peak, current, Ordering::SeqCst, Ordering::SeqCst
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
        
        let mut ips = self.active_ips.write().await;
        if !ips.contains(&ip) {
            ips.push(ip);
        }
    }

    pub async fn connection_closed(&self, ip: String) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
        let mut ips = self.active_ips.write().await;
        ips.retain(|x| x != &ip);
    }

    pub fn add_bytes(&self, bytes: u64) {
        self.total_bytes_transferred.fetch_add(bytes, Ordering::SeqCst);
    }

    pub async fn set_tunnel_status(&self, connected: bool) {
        *self.tunnel_connected.write().await = connected;
    }
}

// ═══════════════════════════════════════════════════════════════════
// Web API
// ═══════════════════════════════════════════════════════════════════

#[derive(Serialize)]
pub struct StatsResponse {
    pub active_connections: usize,
    pub total_bytes_transferred: u64,
    pub tunnel_status: String,
    pub uptime_secs: u64,
    pub peak_connections: usize,
    pub total_connections: u64,
    pub active_ips: Vec<String>,
}

async fn get_stats(axum::extract::State(stats): axum::extract::State<Arc<AppStats>>) -> Json<StatsResponse> {
    Json(StatsResponse {
        active_connections: stats.get_active_connections(),
        total_bytes_transferred: stats.get_total_bytes_transferred(),
        tunnel_status: if stats.get_tunnel_status().await { "connected".into() } else { "disconnected".into() },
        uptime_secs: stats.get_start_time().elapsed().as_secs(),
        peak_connections: stats.get_peak_connections(),
        total_connections: stats.get_total_connections(),
        active_ips: stats.get_active_ips().await,
    })
}

async fn start_web_server(stats: Arc<AppStats>, port: u16) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers(Any);
    
    let app = Router::new()
        .route("/api/stats", get(get_stats))
        .layer(cors)
        .with_state(stats);

    let addr = format!("0.0.0.0:{}", port);
    info!("[web] 🌐 Dashboard API → http://{}", addr);

    // Безопасный bind с обработкой ошибок
    let listener = match TcpListener::bind(&addr).await {
        Ok(ln) => ln,
        Err(e) => {
            error!("[web] ❌ Не удалось запустить веб-сервер на {}: {}", addr, e);
            error!("[web] 💡 Возможно, порт {} уже занят другим процессом", port);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("[web] ❌ Ошибка веб-сервера: {}", e);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Config — загрузка конфигурации
// ═══════════════════════════════════════════════════════════════════

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    mode: String,
    game_port: u16,
    tunnel_port: u16,
    vps_address: String,
    minecraft_address: String,
    reconnect_delay_secs: u64,
    max_reconnect_attempts: u32,
    web_port: u16,
}

impl Config {
    fn load() -> Result<Self, String> {
        // Получаем путь к конфигу из аргументов или используем дефолтный
        let args: Vec<String> = std::env::args().collect();
        let config_path = args.get(1).map(|s| s.as_str()).unwrap_or("config.toml");

        info!("[config] 📄 Загрузка конфига: {}", config_path);

        // Безопасное чтение файла
        let text = match std::fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => {
                return Err(format!(
                    "Не удалось прочитать конфиг '{}': {}\n\
                     💡 Подсказка: скопируйте config.example.toml в config.toml\n\
                     💡 Или укажите путь: ./mc-tunnel /path/to/config.toml",
                    config_path, e
                ));
            }
        };

        // Безопасный парсинг TOML
        match toml::from_str(&text) {
            Ok(config) => Ok(config),
            Err(e) => {
                Err(format!(
                    "Ошибка парсинга конфига '{}': {}\n\
                     💡 Проверьте синтаксис TOML-файла",
                    config_path, e
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bridge — проксирование трафика
// ═══════════════════════════════════════════════════════════════════

async fn bridge(
    mut a: TcpStream,
    mut b: TcpStream,
    label: String,
    stats: Arc<AppStats>,
    player_ip: Option<String>
) {
    if let Some(ref ip) = player_ip {
        stats.connection_opened(ip.clone()).await;
    }

    match io::copy_bidirectional(&mut a, &mut b).await {
        Ok((up, down)) => {
            info!("[{}] закрыт — ↑{} ↓{} байт", label, up, down);
            stats.add_bytes(up + down);
        }
        Err(e) => {
            // Не логируем ConnectionReset/BrokenPipe как ошибки — это норма при дисконнекте
            if e.kind() != std::io::ErrorKind::ConnectionReset 
               && e.kind() != std::io::ErrorKind::BrokenPipe {
                warn!("[{}] разрыв соединения: {}", label, e);
            }
        }
    }

    if let Some(ip) = player_ip {
        stats.connection_closed(ip).await;
    }
}

// ═══════════════════════════════════════════════════════════════════
// VPS Mode — серверная сторона
// ═══════════════════════════════════════════════════════════════════

async fn vps_main(cfg: Arc<Config>, stats: Arc<AppStats>) {
    // Безопасный bind для game порта
    let game_ln = match TcpListener::bind(format!("0.0.0.0:{}", cfg.game_port)).await {
        Ok(ln) => {
            info!("[vps] 🎮 Game listener → 0.0.0.0:{}", cfg.game_port);
            ln
        }
        Err(e) => {
            error!("[vps] ❌ Не удалось открыть game порт {}: {}", cfg.game_port, e);
            error!("[vps] 💡 Проверьте, не занят ли порт: sudo lsof -i :{}", cfg.game_port);
            return;
        }
    };

    // Безопасный bind для tunnel порта
    let tunnel_ln = match TcpListener::bind(format!("0.0.0.0:{}", cfg.tunnel_port)).await {
        Ok(ln) => {
            info!("[vps] 🔌 Tunnel listener → 0.0.0.0:{}", cfg.tunnel_port);
            ln
        }
        Err(e) => {
            error!("[vps] ❌ Не удалось открыть tunnel порт {}: {}", cfg.tunnel_port, e);
            error!("[vps] 💡 Проверьте, не занят ли порт: sudo lsof -i :{}", cfg.tunnel_port);
            return;
        }
    };

    info!("[vps] ✅ VPS запущен, ожидание клиента...");

    loop {
        stats.set_tunnel_status(false).await;

        // Ожидание control-соединения от клиента
        let (mut control, addr) = match tunnel_ln.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("[vps] ❌ Ошибка accept на tunnel: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        // Проверка handshake
        let mut buf = [0u8; 4];
        if let Err(e) = control.read_exact(&mut buf).await {
            warn!("[vps] ⚠ Клиент {} не прислал HELO: {}", addr, e);
            continue;
        }

        if &buf != HELO {
            warn!("[vps] ⚠ Неверный handshake от {}: {:?}", addr, buf);
            continue;
        }

        info!("[vps] ✔ Клиент {} подключился", addr);
        
        if let Err(e) = control.write_all(OKOK).await {
            error!("[vps] ❌ Не удалось отправить OKOK: {}", e);
            continue;
        }

        stats.set_tunnel_status(true).await;

        let control = Arc::new(tokio::sync::Mutex::new(control));

        // Основной цикл обработки игроков
        loop {
            // Ожидание игрока
            let (player_sock, player_addr) = match game_ln.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("[vps] ❌ Ошибка accept на game: {}", e);
                    continue;
                }
            };

            let player_ip = player_addr.ip().to_string();
            info!("[vps] 🎮 Игрок подключился: {}", player_ip);

            // Сигнал клиенту о новом игроке
            {
                let mut ctrl = control.lock().await;
                if let Err(e) = ctrl.write_all(&[SIG_NEW_PLAYER]).await {
                    warn!("[vps] ⚠ Потеряно соединение с клиентом: {}", e);
                    break; // Возвращаемся к ожиданию нового клиента
                }
            }

            // Ожидание data-соединения
            match tunnel_ln.accept().await {
                Ok((mut data_sock, _)) => {
                    let mut b = [0u8; 1];
                    
                    if let Err(e) = data_sock.read_exact(&mut b).await {
                        warn!("[vps] ⚠ Ошибка чтения DATA_HANDSHAKE: {}", e);
                        continue;
                    }

                    if b[0] == DATA_HANDSHAKE {
                        if let Err(e) = data_sock.write_all(&[DATA_ACK]).await {
                            warn!("[vps] ⚠ Ошибка отправки DATA_ACK: {}", e);
                            continue;
                        }

                        let s = stats.clone();
                        let ip = player_ip.clone();
                        tokio::spawn(bridge(
                            player_sock,
                            data_sock,
                            format!("player:{}", ip),
                            s,
                            Some(ip)
                        ));
                    } else {
                        warn!("[vps] ⚠ Неожиданный байт вместо DATA_HANDSHAKE: {}", b[0]);
                    }
                }
                Err(e) => {
                    error!("[vps] ❌ Ошибка accept для data: {}", e);
                }
            }
        }

        info!("[vps] ⏳ Клиент отключился, ожидание переподключения...");
    }
}

// ═══════════════════════════════════════════════════════════════════
// Client Mode — клиентская сторона
// ═══════════════════════════════════════════════════════════════════

async fn client_main(cfg: Arc<Config>, stats: Arc<AppStats>) {
    let mut attempt = 0u32;
    let max_attempts = cfg.max_reconnect_attempts;
    let delay = Duration::from_secs(cfg.reconnect_delay_secs);

    loop {
        attempt += 1;
        
        if max_attempts > 0 && attempt > max_attempts {
            error!("[client] ❌ Превышено максимальное число попыток ({})", max_attempts);
            break;
        }

        info!("[client] 🔌 Подключение к VPS: {} (попытка #{})", cfg.vps_address, attempt);

        // Безопасное подключение к VPS
        let mut control = match TcpStream::connect(&cfg.vps_address).await {
            Ok(stream) => stream,
            Err(e) => {
                warn!("[client] ⚠ Не удалось подключиться: {}", e);
                info!("[client] ⏳ Повтор через {} сек...", cfg.reconnect_delay_secs);
                sleep(delay).await;
                continue;
            }
        };

        // Отправка HELO
        if let Err(e) = control.write_all(HELO).await {
            warn!("[client] ⚠ Ошибка отправки HELO: {}", e);
            sleep(delay).await;
            continue;
        }

        // Ожидание OKOK
        let mut buf = [0u8; 4];
        if let Err(e) = control.read_exact(&mut buf).await {
            warn!("[client] ⚠ Ошибка чтения OKOK: {}", e);
            sleep(delay).await;
            continue;
        }

        if &buf != OKOK {
            warn!("[client] ⚠ Неверный ответ от VPS: {:?}", buf);
            sleep(delay).await;
            continue;
        }

        info!("[client] ✅ Подключено к VPS!");
        stats.set_tunnel_status(true).await;
        attempt = 0; // Сброс счётчика при успешном подключении

        // Основной цикл обработки сигналов от VPS
        loop {
            let mut sig = [0u8; 1];
            
            if let Err(e) = control.read_exact(&mut sig).await {
                warn!("[client] ⚠ Потеряно соединение с VPS: {}", e);
                break;
            }

            if sig[0] != SIG_NEW_PLAYER {
                warn!("[client] ⚠ Неизвестный сигнал: {}", sig[0]);
                continue;
            }

            let vps = cfg.vps_address.clone();
            let mc = cfg.minecraft_address.clone();
            let s = stats.clone();

            tokio::spawn(async move {
                // Data-соединение к VPS
                let mut data = match TcpStream::connect(&vps).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        warn!("[client] ⚠ Не удалось создать data-соединение: {}", e);
                        return;
                    }
                };

                if let Err(e) = data.write_all(&[DATA_HANDSHAKE]).await {
                    warn!("[client] ⚠ Ошибка отправки DATA_HANDSHAKE: {}", e);
                    return;
                }

                let mut ack = [0u8; 1];
                if let Err(e) = data.read_exact(&mut ack).await {
                    warn!("[client] ⚠ Ошибка чтения DATA_ACK: {}", e);
                    return;
                }

                if ack[0] != DATA_ACK {
                    warn!("[client] ⚠ Неверный DATA_ACK: {}", ack[0]);
                    return;
                }

                // Подключение к локальному MC серверу
                let mc_sock = match TcpStream::connect(&mc).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        warn!("[client] ⚠ Не удалось подключиться к MC серверу {}: {}", mc, e);
                        warn!("[client] 💡 Убедитесь, что Minecraft сервер запущен");
                        return;
                    }
                };

                bridge(data, mc_sock, "tunnel".into(), s, None).await;
            });
        }

        stats.set_tunnel_status(false).await;
        info!("[client] ⏳ Переподключение через {} сек...", cfg.reconnect_delay_secs);
        sleep(delay).await;
    }
}

// ═══════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_tunnel=info".parse().unwrap())
                .add_directive("info".parse().unwrap())
        )
        .init();

    // Безопасная загрузка конфига
    let cfg = match Config::load() {
        Ok(config) => {
            info!("[main] ✅ Конфиг загружен успешно");
            info!("[main] 📋 Режим: {}", config.mode);
            Arc::new(config)
        }
        Err(e) => {
            error!("[main] ❌ {}", e);
            std::process::exit(1);
        }
    };

    let stats = Arc::new(AppStats::new());

    // Запуск веб-сервера только в VPS режиме
    if cfg.mode == "vps" {
        let s = stats.clone();
        let p = cfg.web_port;
        tokio::spawn(start_web_server(s, p));
    }

    match cfg.mode.as_str() {
        "vps" => {
            info!("[main] 🚀 Запуск в режиме VPS");
            vps_main(cfg, stats).await;
        }
        "client" => {
            info!("[main] 🏠 Запуск в режиме Client");
            client_main(cfg, stats).await;
        }
        other => {
            error!("[main] ❌ Неизвестный режим: '{}'", other);
            error!("[main] 💡 Допустимые значения: 'vps' или 'client'");
            std::process::exit(1);
        }
    }
}
