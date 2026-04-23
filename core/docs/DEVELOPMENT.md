# Development

---

## Building

```bash
# Full workspace
cargo build --workspace

# Release binaries only
cargo build --release --bins

# Single crate
cargo build -p crawlds-sysmon

# Run daemon directly (no install)
CRAWLDS_LOG=debug cargo run -p crawlds-daemon
```

---

## Running locally

```bash
# Terminal 1 — run daemon
CRAWLDS_LOG=debug cargo run -p crawlds-daemon

# Terminal 2 — use CLI against it
cargo run -p crawlds-cli -- sysmon --cpu
cargo run -p crawlds-cli -- brightness --set=75

# Or use curl directly
SOCK=$XDG_RUNTIME_DIR/crawlds.sock
curl --unix-socket $SOCK http://localhost/health
curl --unix-socket $SOCK http://localhost/sysmon/cpu | jq .
curl --unix-socket $SOCK http://localhost/power/battery | jq .

# Watch the SSE stream
curl --no-buffer --unix-socket $SOCK http://localhost/events
```

---

## Adding a domain

1. Create the crate:
   ```bash
   cargo new --lib crates/crawlds-newdomain
   ```

2. Add to workspace in `Cargo.toml`:
   ```toml
   members = [ ..., "crates/crawlds-newdomain" ]
   ```

3. Implement the interface — every domain crate exposes:
   ```rust
   // Config struct with Default impl (used by figment)
   pub struct Config { ... }

   // Entry point — called by crawlds-daemon, runs indefinitely
   pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()>
   ```

4. Add events to `crawlds-ipc/src/events.rs`:
   ```rust
   pub enum CrawlEvent {
       ...
       NewDomain(NewDomainEvent),
   }
   pub enum NewDomainEvent { ... }
   ```

5. Add types to `crawlds-ipc/src/types.rs` if needed.

6. Wire into daemon:
   - Add dep in `crawlds-daemon/Cargo.toml`
   - Add `Config` field in `crawlds-daemon/src/config.rs`
   - Spawn in `spawn_domains()` in `main.rs`
   - Add routes in `router.rs`

7. Add CLI subcommand in `crawlds-cli/src/cmd/`.

---

## Debugging the socket

```bash
SOCK=$XDG_RUNTIME_DIR/crawlds.sock

# List all routes (health check)
curl -s --unix-socket $SOCK http://localhost/health | jq

# Watch all events, pretty-print JSON
curl --no-buffer --unix-socket $SOCK http://localhost/events \
    | while IFS= read -r line; do
        [[ "$line" == data:* ]] && echo "${line#data: }" | jq --tab .
      done

# Test posting to an endpoint
curl -s --unix-socket $SOCK \
     -X POST -H 'Content-Type: application/json' \
     -d '{"value": 70}' \
     http://localhost/brightness/set | jq

# Check if daemon is running
systemctl --user is-active crawlds
```

---

## Logging

Set `CRAWLDS_LOG` or `RUST_LOG` to control log verbosity:

```bash
CRAWLDS_LOG=debug                       # everything
CRAWLDS_LOG=crawlds_bluetooth=trace,info  # Bluetooth domain verbose, others info
CRAWLDS_LOG=warn,crawlds_notify=debug     # only warnings except notify domain
```

Logs go to the systemd journal when running as a service:

```bash
journalctl --user -u crawlds -f
journalctl --user -u crawlds -f --output=cat   # no metadata prefix
```
