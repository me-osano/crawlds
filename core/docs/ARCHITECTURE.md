# Architecture

High-level view of how crawlds is structured and how data flows through the
daemon, domains, and clients.

---

## Overview

```
crawlds CLI  ──────────────────────────────┐
                                         │  HTTP over Unix socket
crawlds-notify ──┐                         │  $XDG_RUNTIME_DIR/crawlds.sock
crawlds-bluetooth ──────┤                         │
crawlds-network ─────┤   broadcast channel   ──┤──► GET /events  (SSE stream)
crawlds-sysmon ──┤                         │
crawlds-power ───┤                         │
...            │                         │
               └── crawlds-daemon (axum) ──┘
                          │
                   Quickshell QML
                   DataStream / NetworkRequest
```

---

## IPC transport

All communication uses HTTP/1.1 over a Unix domain socket. The wire format is
JSON. This means:

- The CLI is just an HTTP client
- Quickshell's `NetworkRequest` can talk to it natively
- You can debug with `socat` and `curl --unix-socket`
- No custom protocol to maintain

Optional TCP bridge:

If you set `daemon.tcp_addr` in `core.toml`, the daemon also binds a local TCP
listener (e.g. `127.0.0.1:9280`) and serves the same HTTP API + SSE stream.
This is meant for clients that cannot open Unix sockets (some QML builds),
while keeping the Unix socket as the canonical transport.

---

## Event model

Every domain task holds a `tokio::sync::broadcast::Sender<CrawlEvent>`. When
something changes (battery level, Bluetooth device connects, notification
arrives), it sends to that channel. The SSE handler fans events out to all
connected `GET /events` clients as newline-delimited JSON.

---

## Domain isolation

Each domain lives in its own crate (`crawlds-bluetooth`, `crawlds-network`, etc.) with
no dependency on other domains. The only shared surface is `crawlds-ipc`, which
contains the serializable types and event enum. This means:

- Domains are independently testable
- A domain crashing doesn't bring down others (each runs in its own task)
- `crawlds-ipc` can be used in future QML bridge crates or other tools
