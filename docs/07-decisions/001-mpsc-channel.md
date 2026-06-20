# ADR-001: MPSC Channel Between Tokio and ratatui

## Status

Accepted

## Context

ratatui is a synchronous terminal UI framework. Its main loop calls `terminal.draw(|f| render(f, &state))` which blocks until the frame is rendered. However, HFB's core operations (WMI queries, disk cleanup, Windows Update) are asynchronous and potentially long-running.

We need a mechanism to:
1. Run async tasks without blocking the UI render loop
2. Send progress updates from async tasks to the UI
3. Handle user input (keyboard) without blocking async tasks
4. Maintain backpressure to prevent unbounded memory growth

## Decision

Use `tokio::sync::mpsc` (multi-producer, single-consumer) channel with bounded capacity (256 messages) to bridge the async Tokio runtime and the synchronous ratatui loop.

## Consequences

### Positive

- Async tasks never block UI rendering
- UI remains responsive at ~60 FPS
- Backpressure prevents memory exhaustion during fast producers
- Clean separation: UI knows nothing about async runtime details
- Easy to test: mock sender, verify message sequence

### Negative

- Channel capacity limits burst throughput (256 messages)
- Adds complexity: need to handle `send().await` in async tasks
- Message enum `AppMsg` grows large (one variant per event type)
- Requires `tokio::runtime` even in headless mode (can use `block_on`)

## Alternatives Considered

| Alternative | Pros | Cons | Decision |
|-------------|------|------|----------|
| **Callbacks (synchronous)** | Simple, no channel | Blocks async tasks; UI freezes | ❌ Rejected |
| **std::sync::mpsc + block_on** | No Tokio dependency | Deadlock risk; less ergonomic | ❌ Rejected |
| **tokio::sync::broadcast** | Multiple consumers | Overkill; we need single consumer | ❌ Rejected |
| **tokio::sync::watch** | Latest value only | Loses history; bad for logs | ❌ Rejected |
| **tokio::sync::mpsc (bounded)** | Backpressure, clean API | Capacity limit | ✅ Accepted |

## Implementation

```rust
// src/ui/ratatui/mod.rs
let (tx, mut rx) = channel::<AppMsg>(256);

// Async task sends progress
tokio::spawn(async move {
    core::audit::run(tx.clone()).await;
});

// Sync loop receives messages
loop {
    while let Ok(msg) = rx.try_recv() {
        state.update(msg);
    }
    terminal.draw(|f| render(f, &state))?;
}
```

## Related

- [Runtime Architecture](../03-architecture/runtime.md)
- [UI Architecture](../03-architecture/ui.md)

---

*Date: 2026-06-18 | Author: Core Team*
