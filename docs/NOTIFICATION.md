# Notification Architecture

This document describes how notifications and toasts/alerts are implemented in CrawlDS.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           UI Layer                                         │
│  Notification.qml (popup) ← NotificationService.popupModel              │
│  ToastOverlay.qml (toasts) ← ToastService                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    NotificationService (singleton)                         │
│  - Full notification queue management                                      │
│  - Active list + history list                                              │
│  - Progress bars, timeout pause/resume                                    │
│  - Content-based deduplication                                            │
│  - DND state, sounds                                                       │
│  - animateAndRemove signal for UI sync                                    │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CrawlDSService                                          │
│  - notifyEvent signal (from Rust SSE)                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CrawlDS Backend (Rust)                                  │
│  - crawlds-notify: D-Bus notification server + store                     │
│  - SSE events for real-time updates                                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
                         ┌─────────────────────┐
                         │  org.freedesktop   │
                         │  Notifications    │
                         │  (D-Bus)          │
                         └─────────────────────┘
```

## Two Concepts

### 1. Notifications (Full)

System notifications from applications via D-Bus:
- Rich UI with app icon, body, actions
- Can be dismissed, clicked, interacted with
- Stored in history
- Support urgency levels (low, normal, critical)
- Can have action buttons
- Multi-screen support with WlrLayershell positioning

### 2. Toasts/Alerts (Convenience)

Simple feedback messages for user actions:
- Used via `showNotice()`, `showWarning()`, `showError()`
- Shorthand for common notification patterns
- Go through ToastService, not system notifications

---

## Backend (crawlds-notify)

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/notify/list` | GET | List stored notifications |
| `/notify/send` | POST | Send new notification |
| `/notify/:id` | DELETE | Dismiss notification |
| `/notify/action` | POST | Invoke notification action |

### SSE Events

```json
{
  "domain": "notify",
  "data": {
    "event": "new",
    "notification": {
      "id": 123,
      "app_name": "firefox",
      "summary": "Download complete",
      "body": "file.zip has been downloaded",
      "urgency": "normal"
    }
  }
}

{
  "domain": "notify",
  "data": {
    "event": "closed",
    "id": 123
  }
}

{
  "domain": "notify",
  "data": {
    "event": "action",
    "id": 123,
    "action_key": "default"
  }
}
```

---

## NotificationService

Located in `quickshell/services/core/NotificationService.qml`:

### Properties

```qml
property int maxVisible: 5              // Max visible popups
property int maxHistory: 100            // Max history items
property real lastSeenTs: 0             // Last seen timestamp
property bool doNotDisturb: false        // DND state

property ListModel activeList: ListModel {}   // Active notifications
property ListModel historyList: ListModel {}  // Notification history
property ListModel popupModel: ListModel {}   // Alias for activeList (for UI binding)

signal animateAndRemove(string id)     // Signal for UI animation before removal
```

### Functions

```qml
// Dismissal
function dismissActiveNotification(id)     // Dismiss by internal ID
function dismissAllActive()                  // Dismiss all active
function dismissOldestActive()               // Dismiss oldest

// Actions
function invokeAction(id, actionKey)        // Invoke notification action

// Timeout control
function pauseTimeout(id)                   // Pause timeout on hover
function resumeTimeout(id)                  // Resume timeout after hover

// History
function removeFromHistory(id)             // Remove from history
function clearHistory()                     // Clear all history
function getHistorySnapshot()               // Get history as array

// Utility
function findActiveIndex(id)                // Find index in activeList
function urgencyColor(urgency)               // Get color for urgency level
```

### Event Handling

```qml
Connections {
    target: CrawlDSService
    function onNotifyEvent(data) {
        _handleCrawlDSEvent(data)
    }
}

function _handleCrawlDSEvent(data) {
    switch (data.event) {
    case "new":
    case "replaced":
        const notif = _mapCrawlDSNotification(data.notification)
        _handleNewNotification(notif)
        break
    case "closed":
        _handleClosed(data.id)
        break
    }
}
```

---

## UI Components

### Notification.qml

Multi-screen notification popup using `Variants`:
- Binds to `NotificationService.popupModel`
- Uses `WlrLayershell` for wayland positioning
- Supports location setting (top_right, bottom_left, etc.)
- "Dismiss all" bar when 2+ notifications
- Spring animation on height changes

### NotificationCard.qml

Individual notification card with:
- Spring animations (scale, opacity, slide)
- Swipe-to-dismiss (horizontal + vertical based on location)
- Right-click dismiss
- Hover pauses timeout via `pauseTimeout`/`resumeTimeout`
- Progress bar for timed notifications
- Compact/expanded layout modes based on density setting
- Urgency color indicators (left accent bar + progress bar color)

---

## Files Reference

| File | Purpose |
|------|---------|
| `core/crates/crawlds-notify/src/lib.rs` | Backend D-Bus implementation |
| `quickshell/services/core/NotificationService.qml` | Full notification service |
| `quickshell/services/core/ToastService.qml` | Toast/alert functions |
| `quickshell/modules/notification/Notification.qml` | Popup UI |
| `quickshell/modules/notification/NotificationCard.qml` | Card component |
| `quickshell/modules/notification/toast/ToastOverlay.qml` | Toast UI |

---

## Service Separation

### ToastService

Simple toast/alert wrappers (singleton):

```qml
// quickshell/services/core/ToastService.qml
function showNotice(title, body, icon, duration)
function showWarning(title, body, duration)
function showError(title, body, duration)
```

**Use when:** Simple feedback, success messages, warnings, errors.

### NotificationService

Full notification management (singleton):

```qml
// quickshell/services/core/NotificationService.qml
property ListModel popupModel      // Active notifications (for UI binding)
property ListModel historyList     // Notification history

function dismissActiveNotification(id)
function pauseTimeout(id)          // Call on hover
function resumeTimeout(id)         // Call after hover
```

**Use when:** Queue management, progress bars, history, DND, actions, swipe gestures.

---

## Usage Examples

### From Services

```qml
// Show toast
ToastService.showNotice("Battery", "Battery low: 20%", "battery", 6000)
ToastService.showWarning("Bluetooth", "PIN pairing not supported")
ToastService.showError("Download failed", "Network error")
```

### From UI Components (Notifications)

```qml
// Dismiss notification
NotificationService.dismissActiveNotification(id)

// Pause timeout while user hovers (handled automatically by NotificationCard)
NotificationService.pauseTimeout(id)
NotificationService.resumeTimeout(id)

// Listen for removal animation
Connections {
    target: NotificationService
    function onAnimateAndRemove(id) {
        // Handle animation in UI
    }
}
```

---

## Urgency Levels

| Level | Value | Color | Use Case |
|-------|-------|-------|----------|
| low | 0 | overlay1 | Info, success |
| normal | 1 | primary | Standard notifications |
| critical | 2 | error | Errors, warnings |

```qml
function urgencyColor(urgency) {
    switch (urgency) {
    case "critical": return Theme.error
    case "low":      return Color.overlay1
    default:         return Color.primary
    }
}
```

---

## Settings

Notifications can be configured via `Settings.data.notifications`:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `enabled` | bool | true | Enable notifications |
| `monitors` | string[] | all screens | Screens to show on |
| `location` | string | "top_right" | Position (top/bottom + left/right/center) |
| `density` | string | "default" | "compact" or "default" |
| `doNotDisturb` | bool | false | DND state |
| `sounds.enabled` | bool | true | Play sounds |
| `sounds.volume` | number | 0.5 | Sound volume |