# Static-Memory: Technical Standards & Performance Targets

This document serves as a guide for all autonomous agents and developers working on the Static-Memory project.

## 🏗️ Architecture Standards

- **Daemon-Client Model**: Strict separation between the background recorder (daemon) and the interface (client).
- **IPC Transport**:
    - **Linux**: Unix Domain Sockets at \`~/.local/share/static-memory/daemon.sock\`.
    - **Windows**: Named Pipes.
- **Single-Writer Storage**: Only the dedicated storage thread should own the `rusqlite::Connection` and perform writes.

## 🚀 Performance Targets

- **Steady-State RAM**: 50 - 60 MB for the background daemon.
- **TUI Client RAM**: 10 - 15 MB footprint.
- **Binary Size**: ~2.3 MB (Release build).
- **CPU Usage**: < 1% average, even during high input activity.
- **Binary Size**: Optimized via `profile.release` (opt-level = 'z', LTO enabled).

## 🛠️ Implementation Guidelines

- **Allocation Elimination**: Use `SmallVec` and `SmolStr` for hot paths (input capture, window titles, buffer flushing).
- **Error Handling**: Avoid `unwrap()` in production code. Use `expect()` with context or propagate `Result`.
- **Async Hygiene**: Use non-blocking OS APIs (evdev async streams, Win32 message loops in threads).
- **SQLite Optimization**:
    - `PRAGMA journal_mode = WAL`
    - `PRAGMA synchronous = NORMAL`
    - `PRAGMA cache_size = -2000`

## 🧪 Testing Requirements

- All buffer logic must be backed by unit tests in `src/engine/tests.rs`.
- IPC message boundaries must be verified for cross-platform compatibility.
- Ensure terminal raw mode is always restored on exit or panic via the custom panic hook.
