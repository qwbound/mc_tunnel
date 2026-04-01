<<<<<<< HEAD
# mc_tunnel
**Minecraft reverse-proxy tunnel over TCP.** Host a Minecraft server at home without port forwarding — route players through your VPS.
=======
# 🔗 mc-tunnel

**Minecraft reverse-proxy tunnel over TCP.**
Host a Minecraft server at home without port forwarding — route players through your VPS.

**Обратный TCP-туннель для Minecraft.**
Хостите сервер дома без проброса портов — игроки подключаются через ваш VPS.

---

## 📐 Architecture / Архитектура

```
                        ┌──────────── VPS (Linux) ────────────┐
                        │                                      │
  Player ──────────►  :25565 (game)                            │
                        │         ┌────────────────────────┐   │
                        │         │  mc-tunnel (vps mode)  │   │
                        │         └──────────┬─────────────┘   │
                        │                    │                  │
                        │                  :9000 (tunnel)       │
                        └────────────────────┬─────────────────┘
                                             │
                              ◄── Internet ──►
                                             │
                        ┌────────────────────┴─────────────────┐
                        │              Home PC                  │
                        │         ┌────────────────────────┐   │
                        │         │ mc-tunnel (client mode) │   │
                        │         └──────────┬─────────────┘   │
                        │                    │                  │
                        │            127.0.0.1:25565            │
                        │         ┌────────────────────────┐   │
                        │         │   Minecraft Server     │   │
                        │         └────────────────────────┘   │
                        └──────────────────────────────────────┘
```

**Flow / Поток:**

1. **Client** connects to VPS on port `9000` → control channel (HELO/OKOK handshake)
2. **Player** connects to VPS on port `25565`
3. VPS sends signal `0x01` to Client via control channel
4. Client opens a **data** channel to VPS:`9000` (handshake `0x02`/`0x03`)
5. Client opens a connection to local Minecraft `127.0.0.1:25565`
6. VPS bridges player ↔ data channel → `copy_bidirectional`

---

## 🛠 Build / Сборка

### Prerequisites / Требования

- [Rust](https://rustup.rs/) 1.70+ (with `cargo`)

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Compile / Компиляция

```bash
git clone https://github.com/qwbound/mc-tunnel.git
cd mc-tunnel
cargo build --release
```

The binary will be at `./target/release/mc-tunnel`.

---

## 🚀 Setup & Run / Настройка и Запуск

### Step 1 — VPS / Шаг 1 — VPS

```bash
# 1. Copy the binary to your VPS
scp target/release/mc-tunnel user@YOUR_VPS_IP:~/

# 2. SSH into VPS
ssh user@YOUR_VPS_IP

# 3. Create config
cp config.example.toml config.toml
nano config.toml
```

Edit `config.toml` on VPS:
```toml
mode = "vps"
game_port = 25565
tunnel_port = 9000
```

```bash
# 4. Open firewall ports
sudo ufw allow 25565/tcp   # for players
sudo ufw allow 9000/tcp    # for tunnel

# 5. Run
RUST_LOG=info ./mc-tunnel
```

You should see:
```
[vps] 🎮 игроки    → 0.0.0.0:25565
[vps] 🔗 туннель   → 0.0.0.0:9000
[vps] ожидание клиента...
```

### Step 2 — Home PC (Client) / Шаг 2 — Домашний ПК (Клиент)

```bash
# 1. Make sure Minecraft server is running on port 25565

# 2. Create config
cp config.example.toml config.toml
nano config.toml
```

Edit `config.toml` on your PC:
```toml
mode = "client"
vps_address = "YOUR_VPS_IP:9000"
minecraft_address = "127.0.0.1:25565"
reconnect_delay_secs = 3
max_reconnect_attempts = 0
```

```bash
# 3. Run
RUST_LOG=info ./target/release/mc-tunnel
```

You should see:
```
[client] подключение к VPS YOUR_VPS_IP:9000 (попытка 1)…
[client] ✔ control установлен
```

### Step 3 — Connect / Шаг 3 — Подключение

Tell your friends to add server in Minecraft:
```
YOUR_VPS_IP:25565
```

That's it! 🎉

---

## 🔧 Advanced / Дополнительно

### Run as a systemd service / Запуск как systemd-сервис

Create `/etc/systemd/system/mc-tunnel.service`:

```ini
[Unit]
Description=mc-tunnel reverse proxy
After=network.target

[Service]
Type=simple
User=minecraft
WorkingDirectory=/home/minecraft
ExecStart=/home/minecraft/mc-tunnel /home/minecraft/config.toml
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable mc-tunnel
sudo systemctl start mc-tunnel

# Check logs
journalctl -u mc-tunnel -f
```

### Custom config path / Свой путь к конфигу

```bash
./mc-tunnel /etc/mc-tunnel/production.toml
```

### Debug logging / Отладка

```bash
RUST_LOG=debug ./mc-tunnel
```

---

## 📁 Project structure / Структура проекта

```
mc-tunnel/
├── Cargo.toml              # Dependencies
├── config.example.toml     # Example config (safe to commit)
├── config.toml             # Real config (git-ignored!)
├── .gitignore
├── README.md
└── src/
    └── main.rs             # All code (marked with //vps and //client)
```

### Code markers / Маркеры в коде

| Marker     | Meaning                          |
|------------|----------------------------------|
| `//vps`    | VPS-only code                    |
| `//client` | Client-only (home PC) code       |
| no marker  | Shared code (config, bridge, main) |

---

## 📝 TODO / Планы

- [ ] Keep-Alive ping for control channel (prevent idle timeout)
- [ ] TLS encryption for tunnel traffic
- [ ] Authentication token for tunnel connections
- [ ] Multiple client support
- [ ] Web dashboard for monitoring

---

## 📄 License

MIT

---

Made by **qwbound + Claude**
>>>>>>> 4c9b208 (Initial commit: mc-tunnel v0.2 with VPS/Client modes)
