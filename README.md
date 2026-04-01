# ⛏️ MC-Tunnel v0.5

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Tokio-async-blue?logo=rust)](https://tokio.rs/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

**Высокопроизводительный реверс-прокси на Rust для проброса Minecraft-серверов из домашней сети на VPS** — без открытия портов на роутере. Со встроенным мониторингом ресурсов домашнего ПК.

---

## 🌟 Что нового в v0.5

- **🖥️ SysInfo Мониторинг** — CPU, RAM, диски, температура, процессы и сеть домашнего ПК в реальном времени
- **📊 Standalone Dashboard** — полноценный веб-дашборд в одном HTML файле (без React/npm, Tailwind CDN)
- **🔌 Новый API `/api/sysinfo`** — все системные метрики клиента доступны через REST
- **📡 Control Stream** — клиент передаёт sysinfo-данные на VPS через существующий управляющий канал

---

## 🏗️ Архитектура

```
┌──────────────────────────────────────────────────────────────┐
│                        INTERNET                              │
│                                                              │
│    Players ──► VPS:25565 (game_port)                         │
│                  │                                           │
│                  ▼                                           │
│  ┌─────────────────────────────────┐                         │
│  │         VPS (vps mode)          │                         │
│  │                                 │                         │
│  │  ┌──────────┐  ┌────────────┐   │                         │
│  │  │ Game     │  │ Web API    │   │◄── http://vps:3001      │
│  │  │ Listener │  │ :3001      │   │    /api/stats           │
│  │  │ :25565   │  │            │   │    /api/sysinfo         │
│  │  └────┬─────┘  └────────────┘   │                         │
│  │       │                         │                         │
│  │  ┌────▼─────────────────────┐   │                         │
│  │  │ Tunnel Listener :9000   │   │                         │
│  │  │ ┌─────────┐ ┌─────────┐ │   │                         │
│  │  │ │ Control │ │  Data   │ │   │                         │
│  │  │ │ Stream  │ │ Streams │ │   │                         │
│  │  │ └────┬────┘ └────┬────┘ │   │                         │
│  │  └──────┼───────────┼──────┘   │                         │
│  └─────────┼───────────┼──────────┘                         │
│            │           │                                     │
│        TCP tunnel (tunnel_port)                              │
│            │           │                                     │
│  ┌─────────┼───────────┼──────────┐                         │
│  │  Home PC (client mode)         │                         │
│  │         │           │          │                         │
│  │  ┌──────▼────┐ ┌────▼────┐    │                         │
│  │  │ Control   │ │  Data   │    │                         │
│  │  │ + SysInfo │ │ Bridge  │    │                         │
│  │  └───────────┘ └────┬────┘    │                         │
│  │                     │         │                         │
│  │  ┌──────────────────▼──────┐  │                         │
│  │  │  MC Server 127.0.0.1   │  │                         │
│  │  │  :25565                 │  │                         │
│  │  └─────────────────────────┘  │                         │
│  │                               │                         │
│  │  ┌─────────────────────────┐  │                         │
│  │  │  sysinfo 0.30 crate    │  │                         │
│  │  │  CPU / RAM / Disk /    │  │                         │
│  │  │  Temp / Net / Procs    │  │                         │
│  │  └─────────────────────────┘  │                         │
│  └───────────────────────────────┘                         │
└──────────────────────────────────────────────────────────────┘
```

---

## 🚀 Быстрый старт

### 1. Подготовка конфига

```bash
cp config.example.toml config.toml
```

### 2. Настройка `config.toml`

```toml
# ── Mode ────────────────────────────────────────
# "vps"    — запускать на VPS (публичный сервер)
# "client" — запускать на домашнем ПК (хост Minecraft)
mode = "vps"

# ── VPS ─────────────────────────────────────────
game_port = 25565          # Порт для игроков
tunnel_port = 9000         # Порт туннеля (control + data)
web_port = 3001            # Порт веб-дашборда

# ── Client ──────────────────────────────────────
vps_address = "YOUR_VPS_IP:9000"
minecraft_address = "127.0.0.1:25565"

# ── Reconnect ──────────────────────────────────
reconnect_delay_secs = 3
max_reconnect_attempts = 0  # 0 = unlimited
```

### 3. Сборка и запуск

```bash
# На VPS
cargo run --release
# → Tunnel listener на :9000
# → Game listener на :25565
# → Dashboard API на :3001

# На домашнем ПК
cargo run --release
# → Подключается к VPS, пробрасывает MC сервер
# → Собирает sysinfo, шлёт через control stream
```

---

## 📊 Web Dashboard

Дашборд доступен по адресу **`http://ваш-vps:3001`** — файл `dashboard/index.html`.

Это standalone HTML без зависимостей (Tailwind CDN), который опрашивает два API-эндпоинта:

| Метрика | Описание |
|---------|----------|
| 🔵 CPU | Общая нагрузка + разбивка по ядрам |
| 🟢 RAM | Использование + Swap |
| 🟡 Disk | Все подключённые тома |
| 🔴 Temp | Температура CPU |
| 📶 Net | Upload/Download с sparkline-графиком |
| ⚙️ Procs | Топ процессов по CPU/MEM |
| 🔗 Tunnel | Статус, игроки, пик, трафик, IP |

---

## 🔌 API Reference

### `GET /api/stats` (v0.4+)

Статистика туннеля с VPS.

```json
{
  "active_connections": 3,
  "total_bytes_transferred": 15728640,
  "tunnel_status": "connected",
  "uptime_secs": 86400,
  "peak_connections": 8,
  "total_connections": 142,
  "active_ips": ["192.168.1.50", "10.0.0.12"]
}
```

### `GET /api/sysinfo` (v0.5+)

Системная информация домашнего ПК (данные от `sysinfo` crate, переданные через control stream).

```json
{
  "host": {
    "name": "archlinux",
    "os": "Arch Linux",
    "kernel": "6.12.10-arch1-1",
    "arch": "x86_64",
    "uptime_secs": 45230
  },
  "cpu": {
    "model": "AMD Ryzen 5 5600X 6-Core Processor",
    "cores": 6,
    "threads": 12,
    "usage_percent": 23.5,
    "frequency_mhz": 3700,
    "per_core": [12.3, 45.6, 8.1, 67.2, 15.4, 22.8, 9.0, 31.5, 18.7, 42.1, 5.3, 28.9]
  },
  "memory": {
    "total": 34359738368,
    "used": 12884901888,
    "swap_total": 8589934592,
    "swap_used": 0
  },
  "disks": [
    {
      "name": "nvme0n1p2",
      "mount": "/",
      "fs": "ext4",
      "total": 500107862016,
      "used": 156237619200
    }
  ],
  "temperature": {
    "cpu": 52.0,
    "max": 95,
    "label": "k10temp Tctl"
  },
  "network": {
    "interface": "enp0s31f6",
    "rx_bytes": 1073741824,
    "tx_bytes": 536870912
  },
  "processes": {
    "total": 245,
    "list": [
      { "pid": 1234, "name": "java", "cpu": 45.2, "mem": 12.3 },
      { "pid": 5678, "name": "mc-tunnel", "cpu": 2.1, "mem": 0.8 }
    ]
  }
}
```

---

## 🔧 Реализация SysInfo (v0.5)

### 1. Зависимости в `Cargo.toml`

```toml
[dependencies]
sysinfo = "0.30"
```

### 2. Сбор данных в `client_main`

Крейт `sysinfo` работает на **клиенте** (домашний ПК), собирает метрики каждые 2 секунды и отправляет на VPS через **control stream**:

```rust
use sysinfo::System;

// В client_main — фоновая задача
tokio::spawn(async move {
    let mut sys = System::new_all();
    loop {
        sys.refresh_all();
        let payload = collect_sysinfo(&sys);
        // Сериализуем и шлём через control stream
        send_sysinfo_via_control(&control, &payload).await;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
});
```

### 3. Расширение `AppStats`

```rust
pub struct AppStats {
    // ... существующие поля ...

    // v0.5: SysInfo от клиента
    client_sysinfo: RwLock<Option<SysInfoPayload>>,
}
```

### 4. Новый Axum-роут

```rust
// В start_web_server
let app = Router::new()
    .route("/api/stats", get(get_stats))
    .route("/api/sysinfo", get(get_sysinfo))  // NEW
    .layer(cors)
    .with_state(stats);
```

---

## 🗂️ Структура проекта

```
mc_tunnel/
├── Cargo.toml              # Зависимости (tokio, axum, sysinfo, serde, ...)
├── config.example.toml     # Пример конфигурации
├── config.toml             # Рабочий конфиг (в .gitignore)
├── dashboard/
│   └── index.html          # Standalone веб-дашборд (v0.5)
├── src/
│   └── main.rs             # Вся логика: VPS, Client, API, SysInfo
├── README.md
└── CHANGELOG.md
```

---

## 🛠️ Технологии

| Компонент | Технология |
|-----------|------------|
| Runtime | Rust + Tokio (async) |
| Web API | Axum + tower-http CORS |
| System Info | `sysinfo` 0.30 crate |
| Dashboard | Vanilla JS + Tailwind CSS (CDN) |
| Serialization | serde + serde_json + toml |
| Protocol | Custom TCP (HELO/OKOK handshake) |

---

## 🗺️ Roadmap

| Version | Статус | Описание |
|---------|--------|----------|
| v0.1 | ✅ | Базовый TCP туннель |
| v0.2 | ✅ | Автореконнект клиента |
| v0.3 | ✅ | Конфиг через TOML |
| v0.4 | ✅ | Web Dashboard API + IP tracking |
| **v0.5** | 🔧 | **SysInfo мониторинг домашнего ПК** |
| v0.6 | 📋 | Авторизация (токен), TLS |
| v0.7 | 📋 | Мульти-сервер (несколько MC инстансов) |
| v1.0 | 🎯 | Стабильный релиз |

---

## 📄 License

MIT — см. [LICENSE](LICENSE)

---

<p align="center">
  <sub>Made with 🦀 by <a href="https://github.com/qwbound">qwbound</a></sub>
</p>
