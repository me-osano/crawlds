# Quickshell Integration

crawlds is designed to be the backend for a Quickshell QML shell.

---

## Consuming the SSE stream

```qml
// In your Quickshell root or a dedicated Service component
import Quickshell
import Quickshell.Io

pragma Singleton

Singleton {
    id: root

    property real cpuUsage: 0
    property real batteryPercent: 0
    property string batteryState: "unknown"
    property bool onAc: true
    property var notifications: []


    Process {
        id: eventStream
        // Optional TCP bridge: set CRAWLDS_TCP_ADDR=127.0.0.1:9280
        command: {
            const tcp = Quickshell.env("CRAWLDS_TCP_ADDR") || ""
            if (tcp !== "") {
                return ["curl", "--no-buffer", "http://" + tcp + "/events"]
            }
            return ["curl", "--no-buffer",
                    "--unix-socket", Quickshell.env("XDG_RUNTIME_DIR") + "/crawlds.sock",
                    "http://localhost/events"]
        }
        running: true

        stdout: SplitParser {
            onRead: (line) => {
                if (!line.startsWith("data: ")) return
                try {
                    const evt = JSON.parse(line.slice(6))
                    root.handleEvent(evt)
                } catch (_) {}
            }
        }
    }

    function handleEvent(evt) {
        switch (evt.domain) {
        case "sysmon":
            if (evt.data.event === "cpu_update")
                root.cpuUsage = evt.data.cpu.aggregate
            break
        case "power":
            if (evt.data.event === "battery_update") {
                root.batteryPercent = evt.data.status.percent
                root.batteryState   = evt.data.status.state
                root.onAc           = evt.data.status.on_ac
            }
            break
        case "notify":
            if (evt.data.event === "new")
                root.notifications.push(evt.data.notification)
            else if (evt.data.event === "closed")
                root.notifications = root.notifications
                    .filter(n => n.id !== evt.data.id)
            break
        }
    }

    // One-shot HTTP requests to the daemon
    function setBrightness(percent) {
        crawlRequest("POST", "/brightness/set", { value: percent })
    }
    function dismissNotification(id) {
        crawlRequest("DELETE", "/notify/" + id, null)
    }

    function crawlRequest(method, path, body) {
        // TODO: wire up via Quickshell NetworkRequest or a Process curl call
        // NetworkRequest doesn't support Unix sockets natively yet;
        // use Process + curl as a bridge or the CrawlDesktopShell axum bridge crate.
    }
}
```

---

## Bar widget examples

```qml
// Battery widget reading from CrawlDSService
Text {
    text: {
        const pct = CrawlDSService.batteryPercent.toFixed(0)
        const icon = CrawlDSService.onAc ? "\uf084" : "\uf079"
        return icon + " " + pct + "%"
    }
    color: CrawlDSService.batteryPercent < 20 ? "#f38ba8" : "#cdd6f4"
}

// CPU widget
Text {
    text: "\uf61a " + CrawlDSService.cpuUsage.toFixed(1) + "%"
}

```
