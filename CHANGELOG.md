# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.5.0] — 2025-XX-XX

### Added

- **SysInfo мониторинг домашнего ПК** — крейт `sysinfo = "0.30"` собирает метрики на клиенте:
  - CPU: общая нагрузка, нагрузка по ядрам, модель, частота, количество ядер/потоков
  - RAM: используемая / общая, swap
  - Диски: все смонтированные тома (имя, точка монтирования, ФС, объём)
  - Температура: CPU package (значение + максимум + label сенсора)
  - Сеть: TX/RX байт, имя интерфейса, расчёт скорости в реальном времени
  - Процессы: топ по CPU/MEM с PID и именем
  - Хост: hostname, OS, kernel version, архитектура, system uptime
- **Передача sysinfo через control stream** — клиент сериализует данные в JSON и периодически (каждые 2 сек) отправляет на VPS через существующий управляющий канал
- **Новый API эндпоинт `GET /api/sysinfo`** — VPS отдаёт последний snapshot системных данных клиента
- **Расширение `AppStats`** — новое поле `client_sysinfo: RwLock<Option<SysInfoPayload>>` для хранения данных от клиента
- **Фоновая задача `tokio::spawn`** — в `client_main` запускается async-таск для сбора и отправки sysinfo
- **Standalone Web Dashboard** (`dashboard/index.html`):
  - Один HTML-файл, Tailwind CSS через CDN, без React/Vite/npm
  - 4 circular gauge: CPU, RAM, Disk, Temperature (с градиентами и анимацией)
  - Разбивка нагрузки по ядрам CPU с цветовой индикацией
  - Network I/O: Upload/Download со sparkline-графиком
  - CPU & Memory History — 120 точек
  - Storage Volumes — все диски с прогресс-барами
  - Top Processes — таблица (PID, имя, CPU%, MEM%)
  - Tunnel Stats — статус, игроки, пик, трафик, активные IP
  - Тёмная тема с glassmorphism
  - Адаптивный дизайн (mobile-friendly)
- **CHANGELOG.md** — ведение истории изменений

### Changed

- `API_BASE` в дашборде теперь относительный (`window.location.origin`) — работает на любом хосте/порте без хардкода
- Dashboard фетчит два эндпоинта независимо: `/api/stats` (tunnel) и `/api/sysinfo` (system)
- Обновлён `README.md` с полной документацией v0.5, API reference, архитектурной диаграммой

---

## [0.4.0] — 2025-XX-XX

### Added

- **Web Dashboard API** — Axum-сервер на `web_port` с эндпоинтом `GET /api/stats`
- **IP Tracking** — мониторинг подключённых игроков в реальном времени (`active_ips`)
- **Traffic Stats** — учёт `total_bytes_transferred` через `bridge()`
- **Peak Connections** — отслеживание максимального числа одновременных подключений
- **Total Connections** — счётчик всех подключений за всё время работы
- **Uptime** — время работы туннеля
- **CORS** — `tower-http` CORS layer для кроссдоменных запросов к API
- **React Dashboard** (src/) — веб-интерфейс на React + TypeScript + Tailwind:
  - StatCard компоненты
  - StatusBadge
  - TrafficChart
  - ConnectionsTable

### Changed

- **Encapsulated Core** — `AppStats` переписан на геттерах (`get_active_connections()`, `get_tunnel_status()` и т.д.) для безопасного доступа из нескольких tokio-задач
- `connection_opened()` / `connection_closed()` — async-методы с записью IP в `active_ips`
- Фикс IP — теперь берётся `player_addr.ip().to_string()` (чистый IP без порта)

---

## [0.3.0] — 2025-XX-XX

### Added

- **TOML конфигурация** — `config.toml` вместо хардкода:
  - `mode` (vps / client)
  - `game_port`, `tunnel_port`, `web_port`
  - `vps_address`, `minecraft_address`
  - `reconnect_delay_secs`, `max_reconnect_attempts`
- `config.example.toml` — пример конфигурации с комментариями
- Поддержка передачи пути к конфигу через CLI-аргумент

### Changed

- Все параметры (порты, адреса) загружаются из `Config` вместо констант

---

## [0.2.0] — 2025-XX-XX

### Added

- **Автореконнект клиента** — при обрыве соединения клиент переподключается с задержкой (`reconnect_delay_secs`)
- Бесконечный цикл в `client_main` — клиент всегда пытается восстановить связь
- Логирование через `tracing` — структурированные логи с `tracing-subscriber`

### Changed

- Клиент больше не падает при потере контрольного потока — плавный реконнект

---

## [0.1.0] — 2025-XX-XX

### Added

- **Базовый TCP туннель** — VPS ↔ Client через кастомный протокол:
  - Handshake: `HELO` / `OKOK` (4 байта)
  - Сигнал нового игрока: `SIG_NEW_PLAYER` (0x01)
  - Data handshake: `DATA_HANDSHAKE` (0x02) / `DATA_ACK` (0x03)
- **VPS режим** (`vps_main`) — слушает игроков на `game_port`, проксирует через туннель
- **Client режим** (`client_main`) — подключается к VPS, пробрасывает на локальный MC-сервер
- **Bidirectional bridge** — `tokio::io::copy_bidirectional` для проксирования данных
- Асинхронный рантайм на Tokio с `#[tokio::main]`

---

[0.5.0]: https://github.com/qwbound/mc_tunnel/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/qwbound/mc_tunnel/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/qwbound/mc_tunnel/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/qwbound/mc_tunnel/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/qwbound/mc_tunnel/releases/tag/v0.1.0
