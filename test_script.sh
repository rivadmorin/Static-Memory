# Oh I see, tracing spans across awaits are not Send unless we use `instrument` or enter them per future.
# Using `.entered()` ties the span to the thread/local scope and is not safe across `await` points in async Rust.
# We can use `async move { ... }.instrument(span)` instead, or just `let _span = tracing::info_span!("...").entered();` inside a sync block.
# Actually, the proper way to instrument an async block is to do `.instrument(tracing::info_span!(...)).await` on the inner future, or `tracing::info!` instead of a span.
