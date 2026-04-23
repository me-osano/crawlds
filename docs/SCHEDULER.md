# CrawlDS Scheduler

Central scheduler with jitter for timing domain tasks.

## Overview

The scheduler provides coordinated timing for domain tasks with:
- **Jitter**: Random spread to prevent simultaneous wakeups
- **Intervals**: Configurable timing per task
- **Query-based**: Tasks ask "should I run now?"

## Architecture

```
Scheduler ──► TaskRegistry ──► TaskState (per task)
                           ├── interval
                           ├── jitter
                           └── next_run
```

## Usage

### Basic Usage

```rust
use crawlds_scheduler::{Scheduler, ShouldRun};
use std::time::Duration;

let scheduler = Scheduler::new();

// Register a task with interval and jitter
scheduler.register("mytask", Duration::from_secs(1), 0.1).await;

loop {
    // Wait until task is ready (with polling)
    scheduler.wait_interval("mytask", 500).await;
    
    match scheduler.should_run("mytask").await {
        ShouldRun::Yes => {
            // do work
            scheduler.mark_ran("mytask").await;
        }
        ShouldRun::No => {}
    }
}
```

### In Domains

Domains can optionally use the scheduler:

```rust
use crawlds_scheduler::{Scheduler, ShouldRun};
use std::time::Duration;

pub async fn run(cfg: Config, tx: Sender) -> Result<()> {
    run_with_scheduler(cfg, tx, None).await
}

pub async fn run_with_scheduler(
    cfg: Config,
    tx: Sender,
    scheduler: Option<Scheduler>,
) -> Result<()> {
    let task_id = "mytask";
    if let Some(ref sched) = scheduler {
        sched.register(task_id, Duration::from_secs(1), 0.1).await;
    }

    loop {
        if let Some(ref sched) = scheduler {
            sched.wait_interval(task_id, 500).await;
            if sched.should_run(task_id).await == ShouldRun::Yes {
                // do work
                sched.mark_ran(task_id).await;
            } else {
                continue;
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
```

## API Reference

### Scheduler

```rust
pub struct Scheduler {
    tasks: Arc<Mutex<HashMap<&'static str, TaskState>>>,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new()` | Create new scheduler |
| `register(id, interval, jitter_pct)` | Register a task |
| `unregister(id)` | Remove a task |
| `should_run(id)` -> `ShouldRun` | Check if task should run |
| `mark_ran(id)` | Mark task as completed |
| `reset(id)` | Reset task timing |
| `wait_interval(id, poll_ms)` | Wait until task is ready |
| `tasks()` -> `Vec<&str>` | List registered tasks |
| `task_count()` -> `usize` | Number of registered tasks |

### TaskState

```rust
pub struct TaskState {
    pub id: &'static str,
    pub interval: Duration,
    pub jitter: Duration,
    pub next_run: Instant,
    pub last_run: Instant,
}
```

### ShouldRun

```rust
pub enum ShouldRun {
    Yes,
    No,
}
```

## Task Configuration

### Recommended Intervals

| Task | Interval | Jitter | Notes |
|------|----------|--------|-------|
| sysmon | 1s | 5% | High frequency, low jitter |
| network_fast | 5s | 10% | Connectivity checks |
| network_slow | 30s | 15% | WiFi scan, hotspot |
| bluetooth | 5s | 10% | Device scanning |
| clipboard | 500ms | 10% | Frequent changes |
| power | 5s | 10% | Battery status |
| webservice | 60s | 20% | RSS fetch, low priority |

### Understanding Jitter

Jitter adds random spread to prevent simultaneous wakeups:

```
Without jitter (all wake at same time):
|---|---|---|---|---|---|---|---|---|
    ▲ CPU spike

With 10% jitter (spread out):
|---|---|---|---|---|---|---|---|---|
        ▲ smooth
```

- **jitter_pct = 0.1** means ±10% of interval
- For 1s interval: 900ms-1100ms spread
- For 30s interval: 27s-33s spread

## Blocking Variant

For non-async contexts, use `BlockingScheduler`:

```rust
use crawlds_scheduler::BlockingScheduler;

let scheduler = BlockingScheduler::new();
scheduler.register("mytask", Duration::from_secs(1), 0.1);

loop {
    scheduler.wait_until_ready("mytask");
    if scheduler.should_run("mytask") {
        // do work
        scheduler.mark_ran("mytask");
    }
}
```

Enable with feature:
```toml
crawlds-scheduler = { path = "...", features = ["sync"] }
```

## Comparison: With vs Without Scheduler

### Without Scheduler (default)

Each domain runs its own timer:
- Independent intervals
- Potential for simultaneous wakeups
- Simple implementation

### With Scheduler

Central coordination:
- Jitter prevents CPU spikes
- Centralized timing configuration
- More complex domain code

## Implementation Status

| Domain | Status | Notes |
|--------|--------|-------|
| sysmon | ✅ Wired | 1s interval, 5% jitter |
| network | ✅ Wired | 5s (fast) + 30s (slow), 10%/15% jitter |
| clipboard | ✅ Wired | 500ms interval, 10% jitter |
| power | ❌ N/A | Already event-driven via D-Bus |
| bluetooth | ❌ N/A | Already event-driven via D-Bus |
| geolocation | ❌ N/A | Already event-driven via GeoClue2 |
| display | ❌ N/A | Already event-driven via D-Bus |

## File Structure

```
core/crates/crawlds-scheduler/
├── Cargo.toml
└── src/
    ├── lib.rs       # Main entry
    ├── scheduler.rs # Scheduler impl
    ├── task.rs      # TaskState
    └── blocking.rs  # Sync variant
```

## Dependencies

```toml
crawlds-scheduler = { path = "../crawlds-scheduler" }
```

Requires:
- `tokio` (async)
- `fastrand` (jitter generation)

## Troubleshooting

### Task never runs

1. Check registration: `scheduler.tasks().await`
2. Verify interval: `should_run()` returns `No` if not yet time
3. Check jitter: initial delay may be up to jitter% of interval

### High CPU

- Reduce poll interval in `wait_interval()`
- Increase jitter percentage
- Use change detection in domain logic

### Registration conflicts

- Each task needs unique ID
- Cannot re-register same ID without unregistering first
