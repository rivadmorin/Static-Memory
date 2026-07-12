## 2024-05-18 - Mapped Client-Daemon IPC Flow
**Architecture Documentation Gap:** Missing documentation of the IPC communication protocol and module directories.
**Navigation Penalty:** Hard to understand how the TUI client syncs data with the background daemon process without reading the Tokio MPSC and IPC connection source codes.
**Mapping Strategy:** Added ASCII directory tree for codebase structure and Mermaid sequence diagrams for the Client-Daemon IPC message loop in `docs/architecture.md`.