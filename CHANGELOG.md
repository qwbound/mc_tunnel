# Changelog

All notable changes to this project will be documented in this file.

## [0.6.0] - 2025-01-XX

### Added
- **Custom config path**: Now you can specify config file path via command line argument
  ```bash
  ./mc-tunnel /path/to/config.toml
  ```
- **Proper error messages**: All errors now include helpful hints for troubleshooting
- **Peak connections tracking**: Correctly tracks the maximum number of simultaneous connections

### Changed
- **Safe error handling**: Replaced ALL `.unwrap()` calls with proper `match` / `if let` error handling
- **Improved logging**: More informative log messages with emojis for better visibility
- **Graceful degradation**: Application no longer panics on recoverable errors (busy port, network issues, etc.)

### Fixed
- Application crashing when port is already in use
- Silent failures when config file is malformed
- Connection reset errors being logged as critical errors

### Security
- No more panic-based crashes that could leave resources in undefined state

## [0.5.0] - 2024-XX-XX

### Added
- SysInfo monitoring for home PC
- `/api/sysinfo` endpoint
- Standalone web dashboard with Tailwind CSS

## [0.4.0] - 2024-XX-XX

### Added
- Web Dashboard API
- `/api/stats` endpoint
- Active IP tracking
- Peak and total connection counters

## [0.3.0] - 2024-XX-XX

### Added
- TOML configuration file support
- Configurable ports and addresses

## [0.2.0] - 2024-XX-XX

### Added
- Auto-reconnect for client mode
- Configurable reconnect delay and max attempts

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Basic TCP tunnel functionality
- VPS and Client modes
- HELO/OKOK handshake protocol
