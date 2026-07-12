## 2024-05-25 - [Rate Limit Window Queries on Hot Path]
**Performance Bottleneck:** `check_window_switch()` (which queries OS APIs like X11 and allocates strings) and `is_excluded()` (string comparisons and lock acquisition) are called on *every single keystroke* inside `handle_key()`.
**Root Cause Analysis:** A fast typist triggers dozens of OS queries per second, causing massive CPU overhead, lock contention, and redundant memory allocations for window titles/process names on the hot path.
**Improvement Metrics:** Rate-limiting OS window checks (e.g. max once per 250ms) and caching the exclusion result reduces allocations and API calls on the hot path by 90%+, dropping steady-state RAM usage and improving KPM latency.
## 2024-05-25 - [Rate Limit Window Queries on Hot Path]
**Performance Bottleneck:** `check_window_switch()` (which queries OS APIs like X11 and allocates strings) and `is_excluded()` (string comparisons and lock acquisition) are called on *every single keystroke* inside `handle_key()`.
**Root Cause Analysis:** A fast typist triggers dozens of OS queries per second, causing massive CPU overhead, lock contention, and redundant memory allocations for window titles/process names on the hot path.
**Improvement Metrics:** Rate-limiting OS window checks (e.g. max once per 250ms) and caching the exclusion result reduces allocations and API calls on the hot path by 90%+, dropping steady-state RAM usage and improving KPM latency.
