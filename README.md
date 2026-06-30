# Static-Memory

**Static-Memory** is a context-aware local system and activity logger designed for ultra-efficiency and privacy. It runs 100% locally on Windows and Linux Debian.

## Features

- **Global Keylogger**: Captures keystrokes with smart buffering (handles Backspace/Delete).
- **Active Window Tracker**: Associates keystrokes with specific applications and window titles.
- **Immediate Flush & Swap**: Instantly flushes buffers on window change to ensure data integrity.
- **Ultra-Efficient**: Targets 50MB - 60MB RAM usage using Rust, `SmallVec`, and `SmolStr`.
- **TUI Dashboard**: Rich terminal interface built with `ratatui` and `tui-realm` for real-time monitoring and history viewing.
- **Secure Storage**: Uses SQLite with WAL mode and aggressive memory optimizations.
- **Privacy First**: Built-in exclude lists for password managers and private browsing windows.

## Architecture

```text
[Keyboard Hook] ----> [Context Engine] <---- [Window Tracker]
                            |
                    (Immediate Flush)
                            |
               [Tokio MPSC Bounded Channel]
                            |
               [Dedicated Storage Thread]
                            |
                [SQLite (WAL + PRAGMA)]
                            |
                   [Tui-Realm Dashboard]
```

## Installation

### Linux (Debian)
1. Clone the repository.
2. Run the installation script:
   ```bash
   chmod +x install.sh
   ./install.sh
   ```
3. Log out and log back in (to apply `input` group membership).

### Windows
1. Clone the repository.
2. Open PowerShell as Administrator and run:
   ```powershell
   .\install.ps1
   ```

## Configuration

Configuration is stored in `config.toml`. You can exclude specific processes or window titles:

```toml
[privacy]
exclude_processes = ["bitwarden.exe", "keepassxc"]
exclude_titles = ["Incognito", "Private Browsing"]

[storage]
db_path = "activity_log.db"
rotation_size_mb = 50
```

## Storage Schema

| Column | Type | Description |
| --- | --- | --- |
| id | INTEGER | Primary Key |
| timestamp | TEXT | ISO 8601 Timestamp |
| app_name | TEXT | Process name (e.g., chrome.exe) |
| window_title | TEXT | Title of the active window |
| buffer | TEXT | Reconstructed text from keystrokes |

## Memory Optimization

- **SmallVec**: Used for raw event buffering to minimize heap allocations.
- **SmolStr**: Used for persistent strings to save memory on short strings.
- **SQLite PRAGMA**: `cache_size` is limited to ~2MB to stay within strict RAM limits.
- **WAL Mode**: Ensures high performance and durability with minimal overhead.

## License

MIT
