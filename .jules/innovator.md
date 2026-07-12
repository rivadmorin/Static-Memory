## 2024-07-12 - Initial exploration
**Feature Expansion Gap**: Add search history, log export options, and remote backup sync to the TUI client.
**Integration Obstacle**: Need to connect TUI events via IPC and the UI needs modifications. Remote backup doesn't exist at all yet.
**Modular Design Strategy**: TUI modals/components for export already somewhat exist but need to be updated. Remote backup could mean pushing the database to a remote server or similar via IPC message `RemoteBackup`.
## 2024-07-12 - IPC Integration & rusqlite Backup API
**Feature Expansion Gap:** Adding search history and backup features via TUI and IPC daemon.
**Integration Obstacle:** Discovered that `rusqlite::backup::Backup::new` only accepts 2 arguments, not 4 (it does not require `DatabaseName::Main` as the API might differ from raw C APIs or other wrapper versions). 
**Modular Design Strategy:** Updated the modal definitions to cleanly segregate user input handling (like the `SearchModal` input loop) directly within their `tuirealm` mock component structures. Isolated the storage commands within `StorageCommand` so the daemon event loop remains thin.
