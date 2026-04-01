// ╔══════════════════════════════════════════════════════════════════╗
// ║         mc-tunnel v0.6 — Финальная сборка с фиксом IP            ║
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
use tracing::info;

const HELO: &[u8; 4] = b"HELO";
const OKOK: &[u8; 4] = b"OKOK";
const SIG_NEW_PLAYER: u8 = 0x01;
const DATA_HANDSHAKE: u8 = 0x02;
const DATA_ACK: u8 = 0x03;

// //web
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

    pub fn get_active_connections(&self) -> usize { self.active_connections.load(Ordering::SeqCst) }
    pub fn get_total_bytes_transferred(&self) -> u64 { self.total_bytes_transferred.load(Ordering::SeqCst) }
    pub async fn get_tunnel_status(&self) -> bool { *self.tunnel_connected.read().await }
    pub fn get_start_time(&self) -> Instant { self.start_time }
    pub fn get_peak_connections(&self) -> usize { self.peak_connections.load(Ordering::SeqCst) }
    pub fn get_total_connections(&self) -> u64 { self.total_connections.load(Ordering::SeqCst) }
    pub async fn get_active_ips(&self) -> Vec<String> { self.active_ips.read().await.clone() }

    pub async fn connection_opened(&self, ip: String) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
        self.total_connections.fetch_add(1, Ordering::SeqCst);
        let mut ips = self.active_ips.write().await;
        if !ips.contains(&ip) { ips.push(ip); }
    }

    pub async fn connection_closed(&self, ip: String) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
        let mut ips = self.active_ips.write().await;
        ips.retain(|x| x != &ip);
    }

    pub fn add_bytes(&self, bytes: u64) { self.total_bytes_transferred.fetch_add(bytes, Ordering::SeqCst); }
    pub async fn set_tunnel_status(&self, connected: bool) { *self.tunnel_connected.write().await = connected; }
}

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

// //web
async fn start_web_server(stats: Arc<AppStats>, port: u16) {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods([Method::GET]).allow_headers(Any);
    let app = Router::new().route("/api/stats", get(get_stats)).layer(cors).with_state(stats);
    let addr = format!("0.0.0.0:{}", port);
    info!("[web] 🌐 Dashboard API → http://{}", addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    mode: String, game_port: u16, tunnel_port: u16, vps_address: String,
    minecraft_address: String, reconnect_delay_secs: u64, max_reconnect_attempts: u32, web_port: u16,
}

impl Config {
    fn load() -> Self {
        let path = std::env::args().nth(1).unwrap_or("config.toml".into());
        let text = std::fs::read_to_string(&path).unwrap();
        toml::from_str(&text).unwrap()
    }
}

async fn bridge(mut a: TcpStream, mut b: TcpStream, label: String, stats: Arc<AppStats>, player_ip: Option<String>) {
    if let Some(ref ip) = player_ip {
        stats.connection_opened(ip.clone()).await;
    }
    
    if let Ok((up, down)) = io::copy_bidirectional(&mut a, &mut b).await {
        info!("[{label}] закрыт — ↑{up} ↓{down} байт");
        stats.add_bytes(up + down);
    }
    
    if let Some(ip) = player_ip {
        stats.connection_closed(ip).await;
    }
}

// //vps
async fn vps_main(cfg: Arc<Config>, stats: Arc<AppStats>) {
    let game_ln = TcpListener::bind(format!("0.0.0.0:{}", cfg.game_port)).await.unwrap();
    let tunnel_ln = TcpListener::bind(format!("0.0.0.0:{}", cfg.tunnel_port)).await.unwrap();
    loop {
        stats.set_tunnel_status(false).await;
        let (mut control, addr) = tunnel_ln.accept().await.unwrap();
        let mut buf = [0u8; 4];
        if control.read_exact(&mut buf).await.is_ok() && &buf == HELO {
            info!("[vps] ✔ клиент {addr} подключился");
            control.write_all(OKOK).await.unwrap();
            stats.set_tunnel_status(true).await;
            let control = Arc::new(tokio::sync::Mutex::new(control));
            loop {
                let (player_sock, player_addr) = game_ln.accept().await.unwrap();
                if control.lock().await.write_all(&[SIG_NEW_PLAYER]).await.is_err() { break; }
                
                // Ждем DATA_HANDSHAKE
                if let Ok((mut data_sock, _)) = tunnel_ln.accept().await {
                    let mut b = [0u8; 1];
                    if data_sock.read_exact(&mut b).await.is_ok() && b[0] == DATA_HANDSHAKE {
                        data_sock.write_all(&[DATA_ACK]).await.unwrap();
                        let s = stats.clone();
                        let ip = player_addr.ip().to_string(); // Берем чистый IP без порта
                        tokio::spawn(bridge(player_sock, data_sock, format!("player:{}", ip), s, Some(ip)));
                    }
                }
            }
        }
    }
}

// //client
async fn client_main(cfg: Arc<Config>, stats: Arc<AppStats>) {
    loop {
        if let Ok(mut control) = TcpStream::connect(&cfg.vps_address).await {
            control.write_all(HELO).await.unwrap();
            let mut buf = [0u8; 4];
            control.read_exact(&mut buf).await.unwrap();
            stats.set_tunnel_status(true).await;
            loop {
                let mut sig = [0u8; 1];
                if control.read_exact(&mut sig).await.is_err() { break; }
                let vps = cfg.vps_address.clone();
                let mc = cfg.minecraft_address.clone();
                let s = stats.clone();
                tokio::spawn(async move {
                    if let Ok(mut data) = TcpStream::connect(&vps).await {
                        data.write_all(&[DATA_HANDSHAKE]).await.unwrap();
                        let mut ack = [0u8; 1];
                        data.read_exact(&mut ack).await.unwrap();
                        if let Ok(mc_sock) = TcpStream::connect(&mc).await {
                            bridge(data, mc_sock, "tunnel".into(), s, None).await;
                        }
                    }
                });
            }
        }
        stats.set_tunnel_status(false).await;
        sleep(Duration::from_secs(3)).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let cfg = Arc::new(Config::load());
    let stats = Arc::new(AppStats::new());
    if cfg.mode == "vps" {
        let s = stats.clone();
        let p = cfg.web_port;
        tokio::spawn(start_web_server(s, p));
    }
    match cfg.mode.as_str() {
        "vps" => vps_main(cfg, stats).await,
        "client" => client_main(cfg, stats).await,
        _ => (),
    }
}