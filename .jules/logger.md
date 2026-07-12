## 2025-02-12 - Initial Setup
**Telemetry/Logging Defect:** The codebase relies on `println!` for CLI data output and `eprintln!` for errors in `src/main.rs`. We should convert these to use `tracing::info!` and `tracing::error!` with structured JSON for trace contexts. Wait, the `println!` in the CLI output are meant to be stdout for user-facing commands (like `--search` etc.), BUT the instruction specifically mentions: "Replacing raw console.log print statements with structured logging library writes", "Implement structured logs, set request correlation IDs, and redact sensitive variables."

Let's modify the `println!` statements in `src/main.rs` corresponding to `is_search`, `is_top_apps`, `is_total_words`, `is_list_apps`, `is_count_entries`, `is_recent_logs`, `is_busiest_day`, `is_active_hours`. Wait, no, those are strictly CLI output commands! If we look at the instructions, Logger's job is:
- Format system logs into structured JSON payloads that are machine-readable
- Mount unique correlation IDs on requests to trace them across modules
- Redact sensitive user data (passwords, tokens, cards) from logging outputs
- Replacing raw console.log print statements with structured logging library writes

Let's look at `IPCResponse` handlers in `run_daemon` for request tracking. We can add correlation IDs to IPC handling. We also need to add structured logging.

## 2025-02-12 - IPC Spans and Regex Overhead
**Telemetry/Logging Defect:** The initial design created a `tracing::info_span!` and used `.entered()` across an `await` boundary in the Tokio IPC spawn loops.
**Missing/Leaked Log Cause:** Async blocks wrapping `let _span = span.entered();` are not `Send`, causing a compile error when handed to `tokio::spawn()`.
**Standardized Log Structure:** Used `tracing::Instrument` trait: `let response = async { match msg { ... } }.instrument(span).await;` to properly attach the correlation ID context across the entire IPC async handling block.

## 2025-02-12 - Heavy Regex Redaction
**Telemetry/Logging Defect:** Instantiating `regex::Regex::new(...)` inside the hotpath `redact_sensitive_data` caused significant performance overhead and allocations on every keystroke flush.
**Missing/Leaked Log Cause:** Compiling regex at runtime within the engine processing loop.
**Standardized Log Structure:** Implemented `std::sync::OnceLock` to compile the credit card and password redaction regular expressions exactly once upon first invocation.
