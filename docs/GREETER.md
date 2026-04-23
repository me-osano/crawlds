# Greeter Module

CrawlDS Greeter provides a login screen for greetd with support for fingerprint authentication (fprintd), U2F tokens, and PAM-based lockout policies. The system is split between a Rust backend (`crawlds-greeter`) and QML frontend.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BACKEND (Rust)                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-greeter                                                     │   │
│  │                                                                      │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │   │
│  │   │   greetd    │  │    pam      │  │       external         │   │   │
│  │   │   IPC       │  │  detection  │  │  (fprintd/U2F)        │   │   │
│  │   │             │  │             │  │                       │   │   │
│  │   │ • Session   │  │ • Module    │  │ • fprintd probe      │   │   │
│  │   │ • Auth      │  │   detection │  │ • U2F detection       │   │   │
│  │   │ • Launch    │  │ • Lockout   │  │ • D-Bus queries       │   │   │
│  │   └─────────────┘  └─────────────┘  └─────────────────────────┘   │   │
│  │          │                │                    │                 │   │
│  │          └────────────────┼────────────────────┘                 │   │
│  │                           │                                      │   │
│  │   ┌───────────────────────────────────────────────────────────┐   │   │
│  │   │              Memory (session persistence)                   │   │   │
│  │   │   • Last user  • Last session  • Theme preferences        │   │   │
│  │   └───────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-daemon HTTP Router                                           │   │
│  │   GET  /greeter/status           → GreeterManager                   │   │
│  │   POST /greeter/session         → Create greetd session            │   │
│  │   POST /greeter/respond         → Auth response                     │   │
│  │   POST /greeter/cancel         → Cancel session                    │   │
│  │   POST /greeter/launch         → Start session                     │   │
│  │   GET  /greeter/pam-info       → PAM module detection              │   │
│  │   GET  /greeter/external-auth  → fprintd/U2F status               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (QML)                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ GreeterService.qml                                                   │   │
│  │   • HTTP adapter to daemon endpoints                                 │   │
│  │   • PAM info caching                                                │   │
│  │   • External auth status                                            │   │
│  │   • Auth feedback generation                                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ GreeterContent.qml                                                   │   │
│  │   • Login UI (clock, user input, password)                         │   │
│  │   • Wallpaper/background                                            │   │
│  │   • Session selection                                               │   │
│  │   • Power menu                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ GreetdSettings.qml    │  GreetdMemory.qml                           │   │
│  │   • Theme config      │  • Session persistence                        │   │
│  │   • Greeter options   │  • Last user/session                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Backend Crate

**Location**: `core/crates/crawlds-greeter/`

### Modules

- `lib.rs` - Main exports and documentation
- `greetd.rs` - greetd IPC communication
- `pam.rs` - PAM stack detection and parsing
- `external.rs` - fprintd/U2F detection
- `memory.rs` - Session memory persistence
- `types.rs` - Shared types
- `config.rs` - Configuration

### greetd Module

Handles communication with the greetd greeter daemon:

```rust
use crawlds_greeter::{GreeterManager, GreeterSession};

let session = GreeterSession::create("/run/greetd.sock", "username").await?;

match session.respond(Some(password)).await? {
    Response::Success => { /* authenticated */ }
    Response::AuthMessage { msg, type_ } => { /* needs more input */ }
    Response::Error { error_type, msg } => { /* failed */ }
}

session.start(cmd, env).await?;
```

### PAM Module

Detects PAM configuration and lockout policies:

```rust
use crawlds_greeter::pam::PamDetector;

let info = PamDetector::detect_pam_info();
println!("fprintd: {}, U2F: {}, lockout: {}",
    info.has_fprintd, info.has_u2f, info.lockout_configured);

// Generate user-friendly auth feedback
let feedback = PamDetector::generate_auth_feedback(
    "fail",       // pam_state
    2,            // failure_count
    &info,
);
println!("{}", feedback.message); // "Incorrect password - attempt 3 of 10..."
```

### External Auth Module

D-Bus probes for fingerprint scanners:

```rust
use crawlds_greeter::external::ExternalAuthDetector;

let detector = ExternalAuthDetector::new();
let status = detector.detect_external_auth_sync();

if status.available {
    println!("External auth available: {}", status.has_fprintd || status.has_u2f);
}
```

### Memory Module

Session persistence to disk:

```rust
use crawlds_greeter::{memory::Memory, config::Config};

let config = Config::default();
let mut memory = Memory::load(config).await?;

memory.set_last_successful_user(Some("alice".to_string()));
memory.save().await?;

println!("Last user: {:?}", memory.last_successful_user());
```

## HTTP API

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/greeter/status` | GET | Get greeter session status |
| `/greeter/session` | POST | Create greetd session |
| `/greeter/respond` | POST | Send auth response |
| `/greeter/cancel` | POST | Cancel session |
| `/greeter/launch` | POST | Start session |
| `/greeter/pam-info` | GET | Get PAM configuration info |
| `/greeter/external-auth` | GET | Get fprintd/U2F status |

### GET /greeter/pam-info

Get PAM configuration and lockout policy information.

**Response:**
```json
{
    "has_fprintd": true,
    "has_u2f": false,
    "lockout_configured": true,
    "faillock_deny": 3,
    "pam_config_valid": true
}
```

### GET /greeter/external-auth

Get external authentication (fprintd/U2F) status.

**Response:**
```json
{
    "available": true,
    "has_fprintd": true,
    "has_u2f": false,
    "fprintd_probe_complete": true,
    "fprintd_has_device": true
}
```

## Configuration

### Daemon Config

```toml
[greeter]
greetd_socket = "/run/greetd.sock"
session_ttl_secs = 60
```

### Greeter Cache Directory

Settings and memory are stored in `/var/cache/crawlds-greeter/`:

```
/var/cache/crawlds-greeter/
├── settings.json          # Greeter theme settings
├── session.json          # Session config (light/dark mode)
└── .local/state/
    └── memory.json       # Last user/session memory
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `CRAWLDS_GREET_CFG_DIR` | Override greeter cache directory |
| `CRAWLDS_GREET_REMEMBER_LAST_SESSION` | Override session memory |
| `CRAWLDS_GREET_REMEMBER_LAST_USER` | Override user memory |

## QML Services

### GreeterService

Main adapter to the backend:

```qml
// Initialize (loads PAM info from backend)
GreeterService.init()

// Create auth session
GreeterService.createSession("username")

// Send password
GreeterService.respond(password)

// Cancel session
GreeterService.cancelSession()

// Launch session
GreeterService.launch(["wayland-session"], ["DESKTOP_SESSION=gnome"])

// Properties
GreeterService.state              // "inactive", "authenticating", "awaiting_input", "ready", "error"
GreeterService.pamHasFprintd     // PAM fprintd available
GreeterService.pamHasU2f         // PAM U2F available
GreeterService.pamLockoutConfigured  // Lockout policy detected
GreeterService.externalAuthAvailable  // Effective external auth

// Signals
GreeterService.authSuccess()
GreeterService.authFailure(message)
GreeterService.pamInfoLoaded()
GreeterService.externalAuthLoaded()
```

### GreetdSettings

Theme and configuration singleton:

```qml
// Settings
GreetdSettings.weatherEnabled
GreetdSettings.greeterEnableFprint
GreetdSettings.greeterEnableU2f
GreetdSettings.rememberLastUser
GreetdSettings.rememberLastSession
GreetdSettings.greeterWallpaperPath
GreetdSettings.greeterUse24HourClock

// Effective values (with fallbacks)
GreetdSettings.getEffectiveTimeFormat()
GreetdSettings.getEffectiveLockDateFormat()
GreetdSettings.getEffectiveWallpaperFillMode()
```

### GreetdMemory

Session persistence:

```qml
// Get last values
GreetdMemory.lastSuccessfulUser
GreetdMemory.lastSessionId

// Set values
GreetdMemory.setLastSuccessfulUser("alice")
GreetdMemory.setLastSessionId("gnome-xorg")
```

## Greeter Flow

### Login Flow

```
1. User enters username
   └─> GreeterService.createSession(username)

2. If external auth available:
   └─> Auto-start fingerprint/U2F auth
   └─> User presents token

3. User enters password
   └─> GreeterService.respond(password)

4. On success:
   └─> GreeterService.launch(session_cmd, env)

5. On failure:
   └─> Show auth feedback
   └─> Increment failure count
   └─> Check lockout policy
```

### External Auth Flow

```
1. Detect external auth methods:
   └─> GET /greeter/pam-info → has fprintd/U2F
   └─> GET /greeter/external-auth → probe devices

2. If available:
   └─> Show fingerprint/key icon
   └─> User taps token
   └─> greetd responds with empty message

3. Fallback to password:
   └─> If token fails or not enrolled
```

## PAM Lockout

The greeter detects and handles PAM lockout policies:

### Detection

Reads PAM configuration from:
- `/etc/pam.d/greetd`
- `/etc/pam.d/system-auth`
- `/etc/pam.d/common-auth`
- `/etc/security/faillock.conf`

### Feedback Messages

| State | Message |
|-------|---------|
| fail (no lockout) | "Incorrect password" |
| fail (with lockout) | "Incorrect password - attempt X of Y" |
| fail (near lockout) | "Incorrect password - next failures may trigger account lockout" |
| max | "Too many failed attempts - account may be locked" |

## Standalone Binary

The crate includes a standalone binary for greeter-related tasks:

```bash
# Sync settings from user config to greeter cache
crawlds-greeter sync

# Probe PAM configuration
crawlds-greeter probe

# Show greeter status
crawlds-greeter status

# Run as greeter (requires greetd)
crawlds-greeter --socket /run/greetd.sock
```

## Dependencies

### System Dependencies

| Dependency | Required | Purpose |
|------------|----------|---------|
| greetd | Yes | Login manager |
| D-Bus | Yes | IPC |
| pam | Yes | Authentication |
| fprintd | No | Fingerprint auth |

### Rust Crates

- `greetd_ipc` - greetd protocol
- `zbus` - D-Bus client
- `tokio` - Async runtime
- `serde` - Serialization
- `tracing` - Logging

## Troubleshooting

### Greeter not starting

1. Check greetd is running:
   ```bash
   systemctl status greetd
   ```

2. Check socket permissions:
   ```bash
   ls -la /run/greetd.sock
   ```

3. Check greeter user permissions:
   ```bash
   groups greeter
   ```

### External auth not working

1. Check PAM config:
   ```bash
   cat /etc/pam.d/greetd | grep -E 'fprint|u2f'
   ```

2. Probe fprintd:
   ```bash
   crawlds-greeter probe
   ```

3. Check fprintd service:
   ```bash
   systemctl status fprintd
   ```

### Lockout not detected

1. Check PAM configs are readable:
   ```bash
   cat /etc/security/faillock.conf
   ```

2. Check for lockout modules:
   ```bash
   grep -r "pam_faillock\|pam_tally" /etc/pam.d/
   ```

### Session memory not persisting

1. Check cache directory permissions:
   ```bash
   ls -la /var/cache/crawlds-greeter/
   ```

2. Check user ACLs:
   ```bash
   getfacl /home/$USER
   ```

## Security Considerations

- Greeter runs as `greeter` user (not root)
- Socket permissions restrict access to greeter user
- PAM config files are system-protected
- No passwords stored in memory longer than necessary
- Session memory uses atomic writes to prevent corruption

---

## Roadmap / TODOs

### Completed
- [x] `crawlds-greeter` crate created with greetd IPC, PAM detection, external auth
- [x] PAM parsing logic moved from QML to Rust backend (~180 LOC removed from QML)
- [x] HTTP endpoints added: `/greeter/pam-info`, `/greeter/external-auth`
- [x] `GreeterService.qml` updated to use backend endpoints
- [x] `GreeterContent.qml` simplified (removed PAM parsing code)
- [x] Types unified via `crawlds-ipc` (no duplicate `GreeterStatus`, etc.)
- [x] Workspace compiles successfully

### In Progress

### Planned (High Priority)
- [ ] Test PAM detection in actual greeter environment
- [ ] External auth (fprintd/U2F) UI integration in QML
- [ ] Session memory persistence via `crawlds-greeter::Memory`
- [ ] `crawlds-greeter` binary standalone mode for debugging

### Planned (Medium Priority)
- [ ] Wallhaven favorites sync from backend to QML settings
- [ ] OPML import/export for RSS feeds
- [ ] Settings hot-reload without daemon restart

### Testing Checklist
- [ ] PAM module detection works on various distributions
- [ ] External auth detection (fprintd/U2F) works
- [ ] Lockout policy feedback messages display correctly
- [ ] Session persistence remembers last user/session
- [ ] Greeter flow (username → password → launch) works end-to-end
- [ ] Lock screen integration still works (uses Quickshell's built-in PAM)

### Architecture Notes

#### Greeter vs Lock Screen
| Context | Purpose | Implementation |
|---------|---------|----------------|
| Greeter (login) | Detect available PAM modules for UI | `crawlds-greeter` (this crate) |
| Lock screen | Actual PAM authentication | Quickshell's built-in `PamContext` |

**Important**: The lock screen (`LockScreenContent.qml`, `Pam.qml`) uses Quickshell's built-in PAM integration and should NOT be modified. The greeter uses `crawlds-greeter` for UI-related PAM detection only.
