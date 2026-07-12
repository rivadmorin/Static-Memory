## 2023-10-24 - Windows Process Query Access Denied
**Cross-Platform Defect:** Windows `get_active_window` fails to retrieve process names for elevated processes.
**OS Mismatch Cause:** `OpenProcess` was using `PROCESS_QUERY_INFORMATION`, which requires higher privileges than necessary, and `GetModuleBaseNameW` fails if the target process runs as Administrator while the logger does not.
**OS Neutral Strategy:** Refactored to use `PROCESS_QUERY_LIMITED_INFORMATION` combined with `QueryFullProcessImageNameW`, which correctly retrieves the executable path regardless of the target window's UAC elevation level.

## 2023-10-24 - Cross-Platform Unix IPC Socket Extension
**Cross-Platform Defect:** IPC sockets explicitly target `linux`, blocking macOS daemon support.
**OS Mismatch Cause:** macOS also uses Unix Domain Sockets identically to Linux, but the conditional attributes were strictly `#[cfg(target_os = "linux")]`.
**OS Neutral Strategy:** Grouped Unix-like IPC implementation under `#[cfg(any(target_os = "linux", target_os = "macos"))]` into a generalized `unix_ipc` module to easily share POSIX-compliant socket logic.
