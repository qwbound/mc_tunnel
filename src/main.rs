// ╔══════════════════════════════════════════════════════════════════╗
// ║  mc-tunnel v0.2 — Minecraft reverse-proxy tunnel                ║
// ║                                                                  ║
// ║  Маркеры:                                                        ║
// ║    //vps    — код только для VPS                                 ║
// ║    //client — код только для клиента (дом)                       ║
// ║    без маркера — общий код                                       ║
// ╚══════════════════════════════════════════════════════════════════╝

use serde::Deserialize;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

// ── Протокол хэндшейка ──────────────────────────────────────────────
const HELO: &[u8; 4] = b"HELO"; //        control: client → vps
const OKOK: &[u8; 4] = b"OKOK"; //        control: vps → client
const SIG_NEW_PLAYER: u8 = 0x01; //       control: vps → client  "новый игрок"
const DATA_HANDSHAKE: u8 = 0x02; //       data:    client → vps  "я data-канал"
const DATA_ACK: u8 = 0x03; //             data:    vps → client  "принял"

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
}

fn default_game_port() -> u16 { 25565 }
fn default_tunnel_port() -> u16 { 9000 }
fn default_vps_address() -> String { "127.0.0.1:9000".into() }
fn default_mc_address() -> String { "127.0.0.1:25565".into() }
fn default_reconnect_delay() -> u64 { 3 }

impl Config {
    fn load() -> Self {
        let path = std::env::args().nth(1).unwrap_or("config.toml".into());
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Не удалось прочитать {path}: {e}"));
        toml::from_str(&text)
            .unwrap_or_else(|e| panic!("Ошибка парсинга {path}: {e}"))
    }
}

// ── Утилита: двусторонний мост ──────────────────────────────────────
async fn bridge(mut a: TcpStream, mut b: TcpStream, label: &str) {
    match io::copy_bidirectional(&mut a, &mut b).await {
        Ok((up, down)) => info!("[{label}] закрыт — ↑{up} ↓{down} байт"),
        Err(e) => warn!("[{label}] разрыв — {e}"),
    }
}

// ═════════════════════════════════════════════════════════════════════
//vps  ── СЕРВЕРНАЯ ЧАСТЬ (VPS) ────────────────────────────────────────
// ═════════════════════════════════════════════════════════════════════

//vps  Ожидаем control-подключение от клиента, проверяем хэндшейк
//vps  Возвращаем (control_socket, tunnel_listener) — листенер для data
async fn vps_wait_for_client(tunnel_ln: &TcpListener) -> Option<TcpStream> { //vps
    loop { //vps
        let (mut sock, addr) = match tunnel_ln.accept().await { //vps
            Ok(v) => v, //vps
            Err(e) => { //vps
                error!("[vps] accept ошибка: {e}"); //vps
                continue; //vps
            } //vps
        }; //vps

        // Читаем 4 байта хэндшейка //vps
        let mut buf = [0u8; 4]; //vps
        if sock.read_exact(&mut buf).await.is_err() { //vps
            warn!("[vps] {addr} — не прислал хэндшейк, сброс"); //vps
            continue; //vps
        } //vps

        if &buf == HELO { //vps
            info!("[vps] ✔ клиент {addr} подключился (control)"); //vps
            if sock.write_all(OKOK).await.is_err() { //vps
                warn!("[vps] не удалось отправить OKOK"); //vps
                continue; //vps
            } //vps
            return Some(sock); //vps
        } //vps

        warn!("[vps] {addr} — неизвестный хэндшейк {:?}, сброс", buf); //vps
    } //vps
} //vps

//vps  Ожидаем data-подключение от клиента с хэндшейком 0x02
async fn vps_accept_data(tunnel_ln: &TcpListener) -> Option<TcpStream> { //vps
    // Таймаут 10 сек — если клиент не прислал data-канал, сбрасываем //vps
    let deadline = sleep(Duration::from_secs(10)); //vps
    tokio::pin!(deadline); //vps

    loop { //vps
        tokio::select! { //vps
            _ = &mut deadline => { //vps
                warn!("[vps] таймаут ожидания data-канала"); //vps
                return None; //vps
            } //vps
            result = tunnel_ln.accept() => { //vps
                let (mut sock, addr) = match result { //vps
                    Ok(v) => v, //vps
                    Err(e) => { //vps
                        error!("[vps] data accept ошибка: {e}"); //vps
                        continue; //vps
                    } //vps
                }; //vps

                let mut byte = [0u8; 1]; //vps
                if sock.read_exact(&mut byte).await.is_err() { //vps
                    warn!("[vps] {addr} — data: не прислал хэндшейк"); //vps
                    continue; //vps
                } //vps

                if byte[0] == DATA_HANDSHAKE { //vps
                    info!("[vps] ✔ data-канал от {addr}"); //vps
                    let _ = sock.write_all(&[DATA_ACK]).await; //vps
                    return Some(sock); //vps
                } //vps

                warn!("[vps] {addr} — data: неверный байт 0x{:02X}", byte[0]); //vps
            } //vps
        } //vps
    } //vps
} //vps

//vps  Главный цикл VPS
async fn vps_main(cfg: Arc<Config>) { //vps
    let game_addr = format!("0.0.0.0:{}", cfg.game_port); //vps
    let tunnel_addr = format!("0.0.0.0:{}", cfg.tunnel_port); //vps

    let game_ln = TcpListener::bind(&game_addr).await //vps
        .unwrap_or_else(|e| panic!("[vps] bind {game_addr}: {e}")); //vps
    let tunnel_ln = TcpListener::bind(&tunnel_addr).await //vps
        .unwrap_or_else(|e| panic!("[vps] bind {tunnel_addr}: {e}")); //vps

    info!("[vps] 🎮 игроки    → {game_addr}"); //vps
    info!("[vps] 🔗 туннель   → {tunnel_addr}"); //vps

    // Внешний цикл: ждём подключения клиента //vps
    loop { //vps
        info!("[vps] ожидание клиента..."); //vps
        let control = match vps_wait_for_client(&tunnel_ln).await { //vps
            Some(c) => c, //vps
            None => continue, //vps
        }; //vps

        // Arc для того чтобы контрол шарить между тасками (write half) //vps
        let control = Arc::new(tokio::sync::Mutex::new(control)); //vps

        info!("[vps] клиент подключён, слушаем игроков…"); //vps

        // Внутренний цикл: обрабатываем игроков пока control жив //vps
        loop { //vps
            let (player_sock, player_addr) = match game_ln.accept().await { //vps
                Ok(v) => v, //vps
                Err(e) => { //vps
                    error!("[vps] game accept: {e}"); //vps
                    continue; //vps
                } //vps
            }; //vps
            info!("[vps] 🎮 игрок {player_addr}"); //vps

            // Отправляем сигнал клиенту //vps
            { //vps
                let mut ctrl = control.lock().await; //vps
                if ctrl.write_all(&[SIG_NEW_PLAYER]).await.is_err() { //vps
                    warn!("[vps] control разорван, ждём переподключения"); //vps
                    break; // → внешний цикл //vps
                } //vps
            } //vps

            // Ждём data-канал //vps
            let data_sock = match vps_accept_data(&tunnel_ln).await { //vps
                Some(s) => s, //vps
                None => { //vps
                    warn!("[vps] не получили data-канал, дропаем игрока"); //vps
                    continue; //vps
                } //vps
            }; //vps

            // Спариваем //vps
            let label = format!("player:{player_addr}"); //vps
            tokio::spawn(async move { //vps
                bridge(player_sock, data_sock, &label).await; //vps
            }); //vps
        } //vps
    } //vps
} //vps

// ═════════════════════════════════════════════════════════════════════
//client  ── КЛИЕНТСКАЯ ЧАСТЬ (ДОМ) ──────────────────────────────────
// ═════════════════════════════════════════════════════════════════════

//client  Подключение control-канала с хэндшейком
async fn client_connect_control(addr: &str) -> io::Result<TcpStream> { //client
    let mut sock = TcpStream::connect(addr).await?; //client
    sock.write_all(HELO).await?; //client

    let mut buf = [0u8; 4]; //client
    sock.read_exact(&mut buf).await?; //client
    if &buf != OKOK { //client
        return Err(io::Error::new( //client
            io::ErrorKind::InvalidData, //client
            format!("VPS ответил {:?} вместо OKOK", buf), //client
        )); //client
    } //client
    Ok(sock) //client
} //client

//client  Открытие data-канала с хэндшейком 0x02
async fn client_open_data(addr: &str) -> io::Result<TcpStream> { //client
    let mut sock = TcpStream::connect(addr).await?; //client
    sock.write_all(&[DATA_HANDSHAKE]).await?; //client

    let mut ack = [0u8; 1]; //client
    sock.read_exact(&mut ack).await?; //client
    if ack[0] != DATA_ACK { //client
        return Err(io::Error::new( //client
            io::ErrorKind::InvalidData, //client
            format!("VPS data ack = 0x{:02X}", ack[0]), //client
        )); //client
    } //client
    Ok(sock) //client
} //client

//client  Главный цикл клиента с reconnect-логикой
async fn client_main(cfg: Arc<Config>) { //client
    let vps_addr: Arc<str> = Arc::from(cfg.vps_address.as_str()); //client
    let mc_addr: Arc<str> = Arc::from(cfg.minecraft_address.as_str()); //client
    let delay = Duration::from_secs(cfg.reconnect_delay_secs); //client
    let max_attempts = cfg.max_reconnect_attempts; //client

    let mut attempt: u32 = 0; //client

    // Reconnect loop //client
    loop { //client
        attempt += 1; //client
        if max_attempts > 0 && attempt > max_attempts { //client
            error!("[client] исчерпаны попытки ({max_attempts}), выход"); //client
            break; //client
        } //client

        info!("[client] подключение к VPS {vps_addr} (попытка {attempt})…"); //client

        let mut control = match client_connect_control(&vps_addr).await { //client
            Ok(c) => { //client
                info!("[client] ✔ control установлен"); //client
                attempt = 0; // сброс счётчика при успехе //client
                c //client
            } //client
            Err(e) => { //client
                warn!("[client] ✘ не удалось: {e}"); //client
                sleep(delay).await; //client
                continue; //client
            } //client
        }; //client

        // Слушаем сигналы //client
        loop { //client
            let mut sig = [0u8; 1]; //client
            if control.read_exact(&mut sig).await.is_err() { //client
                warn!("[client] control разорван, переподключение…"); //client
                break; //client
            } //client

            if sig[0] != SIG_NEW_PLAYER { //client
                warn!("[client] неизвестный сигнал 0x{:02X}", sig[0]); //client
                continue; //client
            } //client

            info!("[client] ← сигнал: новый игрок"); //client

            let vps = vps_addr.clone(); //  Arc::clone — дёшево //client
            let mc = mc_addr.clone(); //client

            tokio::spawn(async move { //client
                // 1) Data-канал к VPS //client
                let data_sock = match client_open_data(&vps).await { //client
                    Ok(s) => s, //client
                    Err(e) => { //client
                        error!("[client] data-канал: {e}"); //client
                        return; //client
                    } //client
                }; //client

                // 2) Подключение к локальному MC //client
                let mc_sock = match TcpStream::connect(&*mc).await { //client
                    Ok(s) => s, //client
                    Err(e) => { //client
                        error!("[client] minecraft {mc}: {e}"); //client
                        return; //client
                    } //client
                }; //client

                info!("[client] ↔ мост VPS ↔ MC"); //client
                bridge(data_sock, mc_sock, "tunnel").await; //client
            }); //client
        } //client

        warn!("[client] переподключение через {delay:?}…"); //client
        sleep(delay).await; //client
    } //client
} //client

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
    info!("mc-tunnel v0.2 | режим: {}", cfg.mode);

    match cfg.mode.as_str() {
        "vps" => vps_main(cfg).await,       //vps
        "client" => client_main(cfg).await, //client
        other => error!("Неизвестный режим «{other}», укажите mode = \"vps\" | \"client\" в config.toml"),
    }
}
