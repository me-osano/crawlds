Current State:
1. Daemon (json_server.rs): 
   - Unix socket listener that accepts connections
   - Each connection reads commands line-by-line
   - Subscribe flag enables event streaming   - Events are pushed when available   - Simple request/response model
2. Quickshell (CrawlSocket + CrawlDSService):
   - Two CrawlSocket instances (request + subscribe)
   - Exponential backoff reconnect
   - Request tracking with pendingRequests map
   - Response handlers extract .data from responses
Potential Improvements:
1. Protocol Improvements:
   - Add request IDs to ALL requests (currently Subscribe doesn't have one)   - Add proper message framing instead of line-based (handle JSON payloads with newlines)
   - Add ping/pong for connection health checks   - Add request batching or pipelining support
2. Server-Side Improvements:
   - Handle multiple simultaneous connections properly (currently blocks in loop)   - Add connection limits/timeouts
   - Add proper error codes (not just "Err" strings)
   - Add server-side request timeout handling
3. Client-Side Improvements:
   - Add ping/heartbeat mechanism   - Add connection health monitoring
   - Queue requests during disconnect and replay on reconnect   - Add request timeout handling
4. Architecture Improvements:
   - Consider bidirectional communication (daemon can call back to quickshell)
   - Add subscription management (add/remove specific event types)
   - Add service discovery
Let me enumerate a few key improvements in a clear plan format:
1. Add ping/heartbeat - Both sides should periodically ping to verify connection health
2. Add request IDs consistently - All requests should have IDs for tracking
3. Handle reconnection gracefully - Queue pending requests during disconnect and replay
4. Add protocol version negotiation - Allow protocol evolution
5. Consider async request handling - The daemon's synchronous command processing loop is a significant bottleneck that needs addressing
I'll structure this as a proper plan.
Based on my analysis of both json_server.rs (daemon) and CrawlSocket.qml (quickshell), here are the potential improvements:
Current State
Component
Daemon
Quickshell
Improvement Plan
1. Protocol Enhancements:
- Add request IDs consistently (even Subscribe needs one)
- Add ping/pong for connection health checks
- Add protocol version in handshake
2. Connection Handling:
- Add client-side request timeout handling
- Queue pending requests during disconnect, replay on reconnect
- Add connection health monitoring
3. Server Improvements:
- Make request handling non-blocking (spawn async for each command)
- Add request timeouts
- Add proper error codes instead of string errors
4. Subscription Management:
- Currently sends all events to all subscribers
- Could filter by event type (power, network, sysmon, etc.)
---
Questions:
1. Which improvements are most important to you?
2. Should the daemon push events to quickshell (current) or should quickshell poll?
3. Do you want to keep the simplified protocol or add version negotiation?
The biggest win would be adding ping/heartbeat and request timeouts - these address the most common issues with the current setup (silent disconnects, stuck requests).









Comparison: Proposed Approach vs Full JSON-RPC 2.0
Aspect
Request format
Response format
Error format
Batch requests
Notifications
Event format
---
Implementation Effort
Change
Hello handshake
Ping command
Structured errors
Full JSON-RPC
---
### Recommendation: **Proposed (Minimal)**
**Why:**
1. **Minimal change** - works with existing code
2. **Pragmatic** - you already have request IDs
3. **Backward compatible** - existing clients still work
4. **Events stay NDJSON** - streaming works better without RPC overhead
5. **Structured errors** - easier to programmatic handling in quickshell
**Full JSON-RPC would require:**
- Rename `cmd` → `method`
- Add `jsonrpc` field everywhere
- Change response envelope structure
- Update quickshell to match
- No real benefit for event streaming
---
Implementation Plan
1. Daemon changes (json_server.rs):
   - Add Hello command → returns server version, capabilities
   - Add Ping command → returns timestamp
   - Change errors to {code, message} object
2. Quickshell changes (CrawlDSService.qml):
   - On connect, send Hello and store server version
   - Add periodic Ping for connection health
   - Update error handling for structured errors
---
Question for you: Should I proceed with implementing the minimal protocol improvements (Hello, Ping, structured errors), or do you want full JSON-RPC compliance instead?


 opencode
                                   ▄
  █▀▀█ █▀▀█ █▀▀█ █▀▀▄ █▀▀▀ █▀▀█ █▀▀█ █▀▀█
  █  █ █  █ █▀▀▀ █  █ █    █  █ █  █ █▀▀▀
  ▀▀▀▀ █▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀

  Session   New session - 2026-04-20T09:18:44.086Z
  Continue  opencode -s ses_255d0a28affeG7Mrmuv9udfLFT