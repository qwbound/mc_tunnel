# ⛏️ MC-Tunnel v0.6

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Tokio-async-blue?logo=rust)](https://tokio.rs/)
[![License](https://img.shields.io/badge/License-MIT-green)](./LICENSE)

**Высокопроизводительный реверс-прокси на Rust для проброса Minecraft-серверов из домашней сети на VPS** — без открытия портов на роутере.

---

## 🌟 Что нового в v0.6

* **🛡️ Safe Error Handling** — все `.unwrap()` заменены на безопасную обработку ошибок
* **📄 Custom Config Path** — поддержка указания пути к конфигу через аргументы командной строки
* **📊 Улучшенное логирование** — информативные сообщения об ошибках с подсказками
* **🔧 Peak Connections Fix** — корректное отслеживание пикового числа подключений

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
│  │  │ :25565   │  │            │   │                         │
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
│  │  │ Stream    │ │ Bridge  │    │                         │
│  │  └───────────┘ └────┬────┘    │                         │
│  │                     │         │                         │
│  │  ┌──────────────────▼──────┐  │                         │
│  │  │  MC Server 127.0.0.1   │  │                         │
│  │  │  :25565                 │  │                         │
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

# Или с кастомным путём к конфигу:
cargo run --release -- /etc/mc-tunnel/production.toml

# На домашнем ПК
cargo run --release
```

---

## 📄 Кастомный путь к конфигу

Теперь можно указать путь к конфигу через аргументы командной строки:

```bash
# Использование дефолтного config.toml
./mc-tunnel

# Использование кастомного конфига
./mc-tunnel /path/to/my-config.toml
./mc-tunnel ~/configs/vps-config.toml
./mc-tunnel ./configs/client.toml
```

Это полезно когда:
- Нужно запустить несколько инстансов с разными конфигами
- Конфиг хранится в `/etc/` на сервере
- Разные конфиги для dev/prod окружений

---

## 🛡️ Безопасная обработка ошибок (v0.6)

В версии 0.6 все `.unwrap()` заменены на proper error handling:

### Раньше (опасно):
```rust
let listener = TcpListener::bind(&addr).await.unwrap(); // 💥 Паника при ошибке
```

### Теперь (безопасно):
```rust
let listener = match TcpListener::bind(&addr).await {
    Ok(ln) => ln,
    Err(e) => {
        error!("[vps] ❌ Не удалось открыть порт {}: {}", port, e);
        error!("[vps] 💡 Проверьте: sudo lsof -i :{}", port);
        return;
    }
};
```

### Преимущества:
- ✅ Программа не падает при занятом порте
- ✅ Понятные сообщения об ошибках в логах
- ✅ Подсказки по устранению проблем
- ✅ Graceful degradation вместо паники

---

## 📊 Web Dashboard

Дашборд доступен по адресу **`http://ваш-vps:3001`**

### API Endpoints

#### `GET /api/stats`

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

---

## 🗂️ Структура проекта

```
mc_tunnel/
├── Cargo.toml              # Зависимости
├── config.example.toml     # Пример конфигурации
├── config.toml             # Рабочий конфиг (в .gitignore)
├── dashboard/
│   └── index.html          # Веб-дашборд
├── src/
│   └── main.rs             # Вся логика
├── README.md
└── LICENSE
```

---

## 🛠️ Технологии

| Компонент     | Технология                       |
| ------------- | -------------------------------- |
| Runtime       | Rust + Tokio (async)             |
| Web API       | Axum + tower-http CORS           |
| Dashboard     | Vanilla JS + Tailwind CSS (CDN)  |
| Serialization | serde + serde_json + toml        |
| Protocol      | Custom TCP (HELO/OKOK handshake) |

---

## 🗺️ Roadmap

| Version  | Статус | Описание                               |
| -------- | ------ | -------------------------------------- |
| v0.1     | ✅      | Базовый TCP туннель                    |
| v0.2     | ✅      | Автореконнект клиента                  |
| v0.3     | ✅      | Конфиг через TOML                      |
| v0.4     | ✅      | Web Dashboard API + IP tracking        |
| v0.5     | ✅      | SysInfo мониторинг                     |
| **v0.6** | ✅      | **Safe error handling + custom config**|
| v0.7     | 📋     | Авторизация (токен), TLS               |
| v0.8     | 📋     | Мульти-сервер (несколько MC инстансов) |
| v1.0     | 🎯     | Стабильный релиз                       |

---

## 📄 License

MIT — см. [LICENSE](./LICENSE)

---

Made with 🦀 by [qwbound](https://github.com/qwbound)
