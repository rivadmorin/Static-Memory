## 2025-02-12 - Initial Setup
**Telemetry/Logging Defect:** The codebase relies on `println!` for CLI data output and `eprintln!` for errors in `src/main.rs`. We should convert these to use `tracing::info!` and `tracing::error!` with structured JSON for trace contexts. Wait, the `println!` in the CLI output are meant to be stdout for user-facing commands (like `--search` etc.), BUT the instruction specifically mentions: "Replacing raw console.log print statements with structured logging library writes", "Implement structured logs, set request correlation IDs, and redact sensitive variables."

Let's modify the `println!` statements in `src/main.rs` corresponding to `is_search`, `is_top_apps`, `is_total_words`, `is_list_apps`, `is_count_entries`, `is_recent_logs`, `is_busiest_day`, `is_active_hours`. Wait, no, those are strictly CLI output commands! If we look at the instructions, Logger's job is:
- Format system logs into structured JSON payloads that are machine-readable
- Mount unique correlation IDs on requests to trace them across modules
- Redact sensitive user data (passwords, tokens, cards) from logging outputs
- Replacing raw console.log print statements with structured logging library writes

Let's look at `IPCResponse` handlers in `run_daemon` for request tracking. We can add correlation IDs to IPC handling. We also need to add structured logging.
