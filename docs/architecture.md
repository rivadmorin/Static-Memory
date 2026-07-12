# Static-Memory Architecture Maps

## Codebase Directory Structure

```text
src/
├── collector/          # OS-specific input collection (keyboard, window context)
│   ├── keyboard.rs
│   ├── window.rs
│   └── mod.rs
├── engine/             # Core state logic and buffering
│   ├── buffer.rs
│   ├── tests.rs
│   └── mod.rs
├── models/             # Data structures and IPC payloads
│   └── mod.rs
├── os/                 # Platform abstractions and IPC connection layer
│   ├── ipc.rs
│   ├── linux.rs
│   ├── windows.rs
│   └── mod.rs
├── storage/            # SQLite database implementation and background thread
│   ├── db.rs
│   └── mod.rs
├── ui/                 # Thin TUI client implementation
│   ├── app.rs
│   ├── components/
│   │   ├── dashboard.rs
│   │   ├── modals.rs
│   │   └── status_bar.rs
│   └── mod.rs
├── lib.rs
└── main.rs
```

## Client-Daemon IPC Message Flow

The `Static-Memory` application uses a Daemon-Client model where the daemon handles background recording, database interactions, and state, while the thin client connects via Inter-Process Communication (IPC) using Unix Domain Sockets (Linux) or Named Pipes (Windows).

```mermaid
sequenceDiagram
    participant C as TUI Client (Client)
    participant IPC as Local Socket / Pipe
    participant D as Daemon (Server)
    participant E as Core Engine
    participant S as Storage (SQLite)

    %% Connection Phase
    C->>IPC: Attempt Connection (Connect with Retry)
    IPC-->>C: Connection Established
    D->>IPC: Accept Connection

    %% Status Synchronization Loop
    loop Every 2 Seconds
        C->>IPC: Send IPCMessage::GetStatus
        IPC->>D: Receive GetStatus
        D->>E: Query Idle State
        E-->>D: is_idle Boolean
        D->>IPC: Send IPCResponse::Status { is_paused, is_idle }
        IPC-->>C: Receive Status
        C->>C: Update UI State
    end

    %% Timeline Data Request (When Tab 1 is active)
    alt Active Tab is Timeline
        C->>IPC: Send IPCMessage::GetTimeline { limit }
        IPC->>D: Receive GetTimeline
        D->>S: Send StorageCommand::QueryHistory
        S-->>D: Return Vec<LogEntry>
        D->>IPC: Send IPCResponse::Timeline(history)
        IPC-->>C: Receive Timeline Response
        C->>C: Render Timeline Component
    end

    %% Analytics Data Request (When Tab 2 is active)
    alt Active Tab is Dashboard
        C->>IPC: Send IPCMessage::GetAnalytics
        IPC->>D: Receive GetAnalytics
        D->>S: Send StorageCommand::GetAnalytics
        S-->>D: Return AnalyticsData
        D->>IPC: Send IPCResponse::Analytics(data)
        IPC-->>C: Receive Analytics Response
        C->>C: Render Dashboard Component
    end

    %% Export Action
    opt User Triggers Export
        C->>IPC: Send IPCMessage::ExportData { start, end, format }
        IPC->>D: Receive ExportData
        D->>S: Send StorageCommand::Export
        S->>S: Generate File (CSV/TXT) in data dir
        S-->>D: Return Export Status Payload
        D->>IPC: Send IPCResponse::Ok / Error
        IPC-->>C: Acknowledge Export
    end
```