[README_DASHBOARD.md](https://github.com/user-attachments/files/26411492/README_DASHBOARD.md)
# MC-Tunnel Web Dashboard Integration Guide

## 🎯 Обзор изменений

Добавлен Web Dashboard для мониторинга состояния туннеля в реальном времени.

### Новые возможности:
- ✅ REST API для получения статистики (`GET /api/stats`)
- ✅ Отслеживание активных соединений
- ✅ Подсчёт переданного трафика (байты)
- ✅ Статус туннеля (connected/disconnected)
- ✅ Uptime, пиковые и общие соединения
- ✅ CORS поддержка для фронтенда

---

## 📦 Обновление зависимостей

Добавьте в `Cargo.toml`:

```toml
[dependencies]
# ... существующие зависимости ...
axum = "0.7"
tower-http = { version = "0.5", features = ["cors"] }
```

---

## 🏗️ Архитектура

```
┌──────────────────────────────────────────────────────────────┐
│                      MC-TUNNEL v0.3                          │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐    │
│  │   Main      │────►│  AppStats   │◄────│  Bridge     │    │
│  │   Loop      │     │ Arc<...>    │     │  Function   │    │
│  └─────────────┘     └──────┬──────┘     └─────────────┘    │
│                             │                                │
│                             ▼                                │
│                    ┌─────────────────┐                       │
│                    │   Axum Server   │                       │
│                    │   :3001         │                       │
│                    └────────┬────────┘                       │
│                             │                                │
│                             ▼                                │
│                    GET /api/stats                            │
│                    JSON Response                             │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔧 Структура AppStats

```rust
pub struct AppStats {
    /// Количество активных соединений
    pub active_connections: AtomicUsize,
    
    /// Общий трафик в байтах
    pub total_bytes_transferred: AtomicU64,
    
    /// Статус туннеля (true = connected)
    pub tunnel_connected: RwLock<bool>,
    
    /// Время запуска для расчёта uptime
    pub start_time: Instant,
    
    /// Пиковое количество соединений
    pub peak_connections: AtomicUsize,
    
    /// Общее количество соединений за всё время
    pub total_connections: AtomicU64,
}
```

### Методы:
- `connection_opened()` — вызывается при открытии соединения
- `connection_closed()` — вызывается при закрытии соединения
- `add_bytes(bytes)` — добавляет переданные байты
- `set_tunnel_status(bool)` — устанавливает статус туннеля

---

## 🌐 API Endpoint

### GET /api/stats

**Response:**
```json
{
  "active_connections": 5,
  "total_bytes_transferred": 1234567890,
  "tunnel_status": "connected",
  "uptime_secs": 3600,
  "peak_connections": 12,
  "total_connections": 156
}
```

**Headers:**
- `Content-Type: application/json`
- CORS: разрешены все origins (`*`)

---

## ⚙️ Конфигурация

Добавьте в `config.toml`:

```toml
# Порт для Dashboard API
web_port = 3001
```

---

## 🚀 Запуск

### VPS:
```bash
RUST_LOG=info ./mc-tunnel config.toml
```

Вывод:
```
[vps] 🎮 игроки    → 0.0.0.0:25565
[vps] 🔗 туннель   → 0.0.0.0:9000
[web] 🌐 Dashboard API → http://0.0.0.0:3001
[vps] ожидание клиента...
```

### Client:
```bash
RUST_LOG=info ./mc-tunnel config.toml
```

Вывод:
```
[client] подключение к VPS x.x.x.x:9000 (попытка 1)…
[web] 🌐 Dashboard API → http://0.0.0.0:3001
[client] ✔ control установлен
```

---

## 🖥️ Фронтенд Dashboard

Веб-интерфейс доступен в этой же папке. Откройте `index.html` в браузере.

### Функции:
- Отображение статуса туннеля
- Количество активных игроков
- График трафика в реальном времени
- Таблица активных соединений
- Автообновление каждые 2 секунды

### Демо режим:
Если API недоступен, dashboard показывает mock-данные для демонстрации.

---

## 🔐 Безопасность

⚠️ **Важно:** По умолчанию API доступен без авторизации!

Для production рекомендуется:
1. Ограничить доступ через firewall
2. Добавить авторизацию (bearer token)
3. Использовать HTTPS (nginx reverse proxy)

Пример ограничения через ufw:
```bash
# Разрешить только с локального IP
sudo ufw allow from 127.0.0.1 to any port 3001
```

---

## 📝 Changelog v0.3.0

- [x] Добавлена структура `AppStats` с atomic счётчиками
- [x] Интеграция подсчёта трафика в функцию `bridge`
- [x] Axum веб-сервер в отдельном `tokio::spawn`
- [x] Endpoint `GET /api/stats` с JSON ответом
- [x] CORS поддержка для фронтенда
- [x] Новый параметр `web_port` в конфиге
- [x] React Dashboard с визуализацией

---

Made with ❤️ by qwbound + Claude
