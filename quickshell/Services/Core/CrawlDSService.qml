pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

import qs.Common

Singleton {
    id: root

    signal notifyEvent(var data)
    signal greeterEvent(var data)
    signal clipboardEvent(var data)
    signal rssFeedUpdated(var data)
    signal wallhavenResults(var data)
    signal idleEvent(var data)
    signal nightlightEvent(var data)
    signal themeEvent(var data)
    signal wallpaperEvent(var data)

    // ── Connection state ──────────────────────────────────────────────────
    property bool crawldsAvailable: false
    property bool connected: false
    property bool isConnecting: false
    property bool subscribeConnected: false
    readonly property string socketPath: {
        const r = Quickshell.env("XDG_RUNTIME_DIR") || "/run/user/1000"
        return r + "/crawlds.sock"
    }

    signal connectionStateChanged()

    property var pendingRequests: ({})
    property int requestIdCounter: 0

    // ── Battery ───────────────────────────────────────────────────────────
    property real   batteryPercent:    100
    property string batteryState:      "unknown"    // charging | discharging | full | empty
    property bool   batteryOnAc:       true
    property int    batteryTimeToEmpty: 0            // seconds
    property int    batteryTimeToFull:  0            // seconds

    // ── Network ───────────────────────────────────────────────────────────
    property bool   netWifiEnabled:    false
    property bool   netWifiAvailable:  false
    property bool   netEthernetAvailable: false
    property string netConnectivity:   "unknown"    // full | limited | none
    property string netActiveSsid:     ""
    property int    netSignal:         0             // 0-100

    signal netWifiListUpdated(var networks)
    signal netWifiDetailsUpdated(var details)
    signal netEthernetListUpdated(var interfaces)
    signal netEthernetDetailsUpdated(var details)
    signal netWifiScanStarted()
    signal netWifiScanFinished()
    signal netHotspotStatusChanged(var status)
    signal netHotspotStarted(var status)
    signal netHotspotStopped()

    // ── Bluetooth ─────────────────────────────────────────────────────────
    property bool   btPowered:         false
    property bool   optBtPowered:      btPowered   // Optimistic value for immediate UI feedback
    property bool   btDiscovering:     false
    property var    btDevices:         []
    property int    btConnectedCount:  0

    // ── Sysmon ────────────────────────────────────────────────────────────
    property real   cpuUsage:          0
    property var    cpuCores:          []
    property var    cpuFreqMhz:        []
    property real   cpuTempC:          0
    property real   memUsedMib:        0
    property real   memTotalMib:       0
    property real   memPercent:        0
    property real   loadAvg1:          0
    property real   loadAvg5:          0
    property real   loadAvg15:         0
    property var    sysmonDisks:       []
    property var    sysmonNet:         ({})
    property var    sysmonGpu:         null
    property var    procList:          []

    // ── Notifications (count for badge) ───────────────────────────────────
    property int    notifUnread:       0

    // ── Brightness ────────────────────────────────────────────────────────
    property bool   brightnessAvailable: false
    property string brightnessDevice:    ""
    property real   brightnessPercent:   0

    // ── Nightlight ───────────────────────────────────────────────────────
    property bool nightlightEnabled:  false
    property bool nightlightAvailable: false
    property int  nightlightTemperature: 6500

    // ── VFS / Disk ───────────────────────────────────────────────────────────
    property var    diskUsage:           []
    property var    removableDevices:    []
    signal fileSearchResults(var results)

    // ── Wallpaper ───────────────────────────────────────────────────────────
    property var    wallpaperBackends:      []
    property bool   wallpaperSwwwAvailable: false
    property bool   wallpaperSwwwRunning:   false

    // ── System Info ─────────────────────────────────────────────────────────
    property string compositorName:        ""
    property var    compositorCapabilities: ({})
    property bool   supportsWallpaperControl: false
    property bool   supportsBlur:           false
    property bool   supportsLayerShell:     false

    // ── Internal ──────────────────────────────────────────────────────────
    property var _pendingInit: 0

    // ── Startup ───────────────────────────────────────────────────────────
    Component.onCompleted: {
        if (socketPath && socketPath.length > 0) {
            testProcess.running = true
        }
    }

    Process {
        id: testProcess
        command: ["test", "-S", root.socketPath]

        onExited: exitCode => {
            if (exitCode === 0) {
                root.crawldsAvailable = true
                connectSocket()
            } else {
                root.crawldsAvailable = false
            }
        }
    }

    function connectSocket() {
        if (!root.crawldsAvailable || root.connected || isConnecting) return
        isConnecting = true
        requestSocket.connected = true
    }

    CrawlSocket {
        id: requestSocket
        path: socketPath
        connected: false

        onConnectionStateChanged: {
            if (connected) {
                root.connected = true
                root.isConnecting = false
                Logger.i("CrawlDSService", "Connected to socket")
                connectionStateChanged()
                subscribeSocket.connected = true
            } else {
                root.connected = false
                root.isConnecting = false
                root._resetState()
                Logger.i("CrawlDSService", "Disconnected from socket")
                connectionStateChanged()
            }
        }

        parser: SplitParser {
            onRead: line => {
                if (!line || line.length === 0) return
                let response
                try {
                    response = JSON.parse(line)
                } catch (e) {
                    console.warn("CrawlDSService: Failed to parse response:", line.substring(0, 100))
                    return
                }
                handleResponse(response)
            }
        }
    }

    CrawlSocket {
        id: subscribeSocket
        path: socketPath
        connected: false

        onConnectionStateChanged: {
            root.subscribeConnected = connected
            if (connected) {
                sendHello()
            }
        }

        parser: SplitParser {
            onRead: line => {
                if (!line || line.length === 0) return
                let response
                try {
                    response = JSON.parse(line)
                } catch (e) {
                    console.warn("CrawlDSService: Failed to parse event:", line.substring(0, 100))
                    return
                }
                console.log("CrawlDSService: Subscribe socket <<", line)
                handleSubscriptionEvent(response)
            }
        }
    }

    function sendSubscribeRequest() {
        console.log("CrawlDSService: Subscribing to events")
        subscribeSocket.send({ jsonrpc: "2.0", method: "Subscribe" })
    }

    function sendHello() {
        sendRequest("Hello", { client: "quickshell", version: "1.0" }, (result) => {
            if (result) {
                root.serverVersion = result.version || ""
                Logger.i("CrawlDSService", "Connected to server v" + root.serverVersion)
                sendSubscribeRequest()
                _bootstrap()
            }
        })
    }

    function sendRequest(method, params, callback) {
        if (!connected) {
            console.warn("CrawlDSService.sendRequest: Not connected, method:", method)
            if (callback) callback({ error: "not connected" })
            return
        }

        requestIdCounter++
        const id = requestIdCounter
        const request = { jsonrpc: "2.0", method: method, id: id }
        if (params && Object.keys(params).length > 0) {
            request.params = params
        }

        console.log("CrawlDSService: Sending request id=" + id + " method=" + method)
        if (callback) pendingRequests[id] = callback

        requestSocket.send(request)
    }

    function _resetState() {
        root.connected = false
        root.isConnecting = false
        root.subscribeConnected = false
        root.pendingRequests = ({})
        root.serverVersion = ""
    }

    property string serverVersion: ""

    function handleResponse(response) {
        const callback = pendingRequests[response.id]
        if (callback) {
            delete pendingRequests[response.id]
            if (response.error) {
                console.warn("CrawlDSService: Error", response.error.code, response.error.message)
                callback(null)
            } else if (response.result !== undefined) {
                callback(response.result)
            } else {
                callback(response.result)
            }
        }
    }

    function handleSubscriptionEvent(response) {
        if (response.method === "event" && response.params) {
            _dispatchJson(response.params)
        } else if (response.result && response.result.subscribed) {
            // Subscription confirmed
            console.log("CrawlDSService: Subscribed to events")
        }
    }

    function init() {
        Logger.i("CrawlDSService", "Using Unix socket at " + socketPath)
    }

    function _bootstrap() {
        jsonCmd({ cmd: "PowerBattery" }, _handleBattery)
        jsonCmd({ cmd: "NetStatus" }, _handleNet)
        jsonCmd({ cmd: "BtStatus" }, _handleBt)
        jsonCmd({ cmd: "SysmonCpu" }, _handleCpu)
        jsonCmd({ cmd: "SysmonMem" }, _handleMem)
        jsonCmd({ cmd: "SysmonDisk" }, _handleSysmonDisk)
        jsonCmd({ cmd: "SysmonNet" }, _handleSysmonNet)
        jsonCmd({ cmd: "SysmonGpu" }, _handleSysmonGpu)
        jsonCmd({ cmd: "VfsDiskUsage" }, _handleDiskUsage)
        jsonCmd({ cmd: "DiskList" }, _handleRemovableDevices)
        jsonCmd({ cmd: "BrightnessStatus" }, _handleBrightness)
        jsonCmd({ cmd: "WallpaperBackends" }, _updateWallpaperBackends)
        jsonCmd({ cmd: "SystemInfo" }, _updateSystemInfo)
    }

    function _setCoreConfig(section, key, value) {
        const setCmd = { cmd: "Set", key: section + "." + key, value: value }
        jsonCmd(setCmd, null)
    }

    function _syncCoreSettings(config) {
    }

    function setServiceState(service, state) {
        Qt.createQmlObject(`
            import QtQuick
            import Quickshell.Io

            Process {
                command: ["gdbus", "call", "-e", "-d", "org.crawlds.Config", "-o", "/org/crawlds/Config", "-m", "org.crawlds.Config.Set" + "${service.charAt(0).toUpperCase() + service.slice(1)}State", "-t", "${state}"]
                running: true
            }
        `, root)
    }

    // ── JSON command helper ──────────────────────────────────────────────
    function jsonCmd(cmd, handler) {
        const cmdName = cmd.cmd
        const params = Object.assign({}, cmd)
        delete params.cmd
        const wrappedHandler = handler ? (response) => {
            if (response && response.res === "Ok") {
                handler(response.data)
            } else if (response && response.res === "Err") {
                console.warn("CrawlDSService: Error:", response.error)
                handler(null)
            } else {
                handler(null)
            }
        } : null
        sendRequest(cmdName, Object.keys(params).length > 0 ? params : null, wrappedHandler)
    }

// ── JSON Event dispatcher ─────────────────────────────────────────────
    function _dispatchJson(evt) {
        const domain = evt.domain
        const eventData = evt.data
        // JSON events have .data containing {event, ...}
        switch (domain) {
        case "power":     _handlePowerEvent(eventData);    break
        case "network":   _handleNetEvent(eventData);      break
        case "bluetooth": _handleBtEvent(eventData);       break
        case "sysmon":    _handleSysmonEvent(eventData);   break
        case "notify":    _handleNotifyEvent(eventData); break
        case "brightness":_handleBrightnessEvent(eventData); break
        case "disk":      _handleVfsEvent(eventData);      break
        case "greeter":   greeterEvent(eventData);         break
        case "clipboard": clipboardEvent(eventData);        break
        case "webservice": _handleWebserviceEvent(eventData); break
        case "idle":      idleEvent(eventData);             break
        case "nightlight": nightlightEvent(eventData);      break
        case "theme":     themeEvent(eventData);            break
        case "wallpaper": _handleWallpaperEvent(eventData); break
        }
    }

    // ── Domain handlers ───────────────────────────────────────────────────

    function _handleBattery(data) {
        root.batteryPercent    = data.percent         ?? root.batteryPercent
        root.batteryState      = data.state           ?? root.batteryState
        root.batteryOnAc       = data.on_ac           ?? root.batteryOnAc
        root.batteryTimeToEmpty= data.time_to_empty_secs ?? 0
        root.batteryTimeToFull = data.time_to_full_secs  ?? 0
    }

    function _handlePowerEvent(data) {
        switch (data.event) {
        case "battery_update": _handleBattery(data.status); break
        case "ac_connected":   root.batteryOnAc = true;  break
        case "ac_disconnected":root.batteryOnAc = false; break
        }
    }

    function _handleNet(data) {
        root.netConnectivity = data.connectivity  ?? root.netConnectivity
        root.netWifiEnabled  = data.wifi_enabled  ?? root.netWifiEnabled
        root.netWifiAvailable = data.wifi_available ?? root.netWifiAvailable
        root.netEthernetAvailable = data.ethernet_available ?? root.netEthernetAvailable
        root.netActiveSsid   = data.active_ssid   ?? ""
    }

    function _handleNetEvent(data) {
        switch (data.event) {
        case "connected":
            root.netConnectivity = "full"
            root.netActiveSsid   = data.ssid ?? ""
            break
        case "disconnected":
            root.netConnectivity = "none"
            root.netActiveSsid   = ""
            break
        case "connectivity_changed":
            root.netConnectivity = data.state ?? "unknown"
            break
        case "wifi_enabled":  root.netWifiEnabled = true;  break
        case "wifi_disabled": root.netWifiEnabled = false; break
        case "wifi_list_updated":
            root.netWifiListUpdated(data.networks)
            break
        case "wifi_scan_started":
            root.netWifiScanStarted()
            break
        case "wifi_scan_finished":
            root.netWifiScanFinished()
            break
        case "active_wifi_details_changed":
            root.netWifiDetailsUpdated(data.details)
            break
        case "ethernet_interfaces_changed":
            root.netEthernetListUpdated(data.interfaces)
            break
        case "active_ethernet_details_changed":
            root.netEthernetDetailsUpdated(data.details)
            break
        case "hotspot_status_changed":
            root.netHotspotStatusChanged(data.status)
            break
        case "hotspot_started":
            root.netHotspotStarted(data.status)
            break
        case "hotspot_stopped":
            root.netHotspotStopped()
            break
        }
    }

    function _handleBt(data) {
        root.btPowered      = data.powered      ?? root.btPowered
        root.btDiscovering  = data.discovering  ?? root.btDiscovering
        root.btDevices      = data.devices      ?? []
        root.btConnectedCount = (data.devices ?? []).filter(d => d.connected).length
    }

    function _handleBtEvent(data) {
        switch (data.event) {
        case "adapter_powered": root.btPowered = data.on; break
        case "device_connected":
        case "device_disconnected":
            jsonCmd({ cmd: "BtStatus" }, _handleBt)
            break
        case "scan_started": root.btDiscovering = true;  break
        case "scan_stopped": root.btDiscovering = false; break
        }
    }

    function _handleCpu(data) {
        root.cpuUsage = data.aggregate  ?? 0
        root.cpuCores = data.cores      ?? []
        root.cpuFreqMhz = data.frequency_mhz ?? []
        root.cpuTempC = data.temperature_c ?? 0
        root.loadAvg1 = data.load_avg?.one ?? 0
        root.loadAvg5 = data.load_avg?.five ?? 0
        root.loadAvg15 = data.load_avg?.fifteen ?? 0
    }

    function _handleMem(data) {
        root.memUsedMib  = (data.used_kb  ?? 0) / 1024
        root.memTotalMib = (data.total_kb ?? 0) / 1024
        root.memPercent  = root.memTotalMib > 0
            ? (root.memUsedMib / root.memTotalMib * 100)
            : 0
    }

    function _handleSysmonEvent(data) {
        switch (data.event) {
        case "cpu_update": _handleCpu(data.cpu); break
        case "mem_update": _handleMem(data.mem); break
        case "net_update": _handleSysmonNet(data.traffic); break
        case "gpu_update": _handleSysmonGpu(data.gpu); break
        }
    }

    function _handleSysmonDisk(data) {
        root.sysmonDisks = Array.isArray(data) ? data : []
    }

    function _handleSysmonNet(data) {
        root.sysmonNet = data || {}
    }

    function _handleSysmonGpu(data) {
        root.sysmonGpu = data || null
    }

    function _handleProcList(data) {
        root.procList = Array.isArray(data) ? data : []
    }

    function _handleNotifyEvent(data) {
        switch (data.event) {
        case "new":      root.notifUnread++; break
        case "closed":   root.notifUnread = Math.max(0, root.notifUnread - 1); break
        }
        root.notifyEvent(data)
    }

    function _handleBrightness(data) {
        if (!data) return
        root.brightnessAvailable = true
        root.brightnessDevice = data.device ?? root.brightnessDevice
        root.brightnessPercent = data.percent ?? root.brightnessPercent
    }

    function _handleBrightnessEvent(data) {
        if (!data) return
        switch (data.event) {
        case "changed":
            _handleBrightness(data.status)
            break
        }
    }

    function _handleNightlight(data) {
        if (!data) return
        root.nightlightAvailable = data.available ?? root.nightlightAvailable
        root.nightlightEnabled = data.enabled ?? root.nightlightEnabled
        root.nightlightTemperature = data.temperature_k ?? root.nightlightTemperature
    }

    function _handleNightlightEvent(data) {
        if (!data) return
        switch (data.event) {
        case "enabled":   root.nightlightEnabled = true;  break
        case "disabled":  root.nightlightEnabled = false; break
        case "temperature_changed":
            root.nightlightTemperature = data.temperature_k ?? root.nightlightTemperature
            break
        }
    }

    function _handleDiskUsage(data) {
        root.diskUsage = Array.isArray(data) ? data : []
    }

    function _handleRemovableDevices(data) {
        root.removableDevices = Array.isArray(data) ? data : []
    }

    function _handleVfsEvent(data) {
        switch (data.event) {
        case "disk_usage_updated":
            _handleDiskUsage(data.usage)
            break
        case "device_added":
            _handleRemovableDevices(data.device)
            break
        case "device_removed":
            jsonCmd({ cmd: "DiskList" }, _handleRemovableDevices)
            break
        case "fs_changed":
            root.fsChanged(data.fs_event)
            break
        }
    }

    signal fsChanged(var event)

    // ── Control helpers ───────────────────────────────────────────────────
    function refreshSysmonDisk() {
        jsonCmd({ cmd: "SysmonDisk" }, _handleSysmonDisk)
    }

    function refreshSysmonNet() {
        jsonCmd({ cmd: "SysmonNet" }, _handleSysmonNet)
    }

    function refreshSysmonGpu() {
        jsonCmd({ cmd: "SysmonGpu" }, _handleSysmonGpu)
    }

    function refreshProcList(sortBy, top) {
        const sortKey = sortBy || "cpu"
        const topCount = top || 30
        jsonCmd({ cmd: "ProcList", sort: sortKey, top: topCount }, _handleProcList)
    }

    function setBrightness(pct)      { jsonCmd({ cmd: "BrightnessSet", value: pct }, null) }
    function increaseBrightness(pct) { jsonCmd({ cmd: "BrightnessInc", value: pct }, null) }
    function decreaseBrightness(pct) { jsonCmd({ cmd: "BrightnessDec", value: pct }, null) }

    // ── Nightlight control ─────────────────────────────────────────────────
    function nightlightEnable()  { jsonCmd({ cmd: "NightlightEnable" }, null) }
    function nightlightDisable() { jsonCmd({ cmd: "NightlightDisable" }, null) }
    function nightlightSetTemperature(kelvin) { jsonCmd({ cmd: "NightlightSet", kelvin: kelvin }, null) }

    function dismissNotif(id)        { jsonCmd({ cmd: "NotifyDismiss", id: id }, null) }

    // ── Bluetooth control ───────────────────────────────────────────────────────
    function setBtPowered(on)     { root.optBtPowered = on; jsonCmd({ cmd: "BtPower", enabled: on }, null) }
    function startBtScan()         { jsonCmd({ cmd: "BtScan" }, null) }
    function setBtDiscoverable(on){ jsonCmd({ cmd: "BtDiscoverable", enabled: on }, null) }
    function setBtPairable(on)     { jsonCmd({ cmd: "BtPairable", enabled: on }, null) }

    function connectBtDevice(address)  { jsonCmd({ cmd: "BtConnect", address: address }, null) }
    function disconnectBtDevice(address) { jsonCmd({ cmd: "BtDisconnect", address: address }, null) }
    function pairBtDevice(address) { jsonCmd({ cmd: "BtPair", address: address }, null) }
    function forgetBtDevice(address) { jsonCmd({ cmd: "BtRemove", address: address }, null) }
    function setBtTrusted(address, trusted) { jsonCmd({ cmd: "BtTrust", address: address }, null) }

    // ── Power profile control ─────────────────────────────────────────────
    function setPowerProfile(profile) { jsonCmd({ cmd: "PowerProfileSet", profile: profile }, null) }

    // ── Webservice (RSS, Wallhaven) ─────────────────────────────────────
    function _handleWebserviceEvent(data) {
        switch (data.event) {
        case "rss_feed_updated":
            rssFeedUpdated(data)
            break
        case "wallhaven_results":
            wallhavenResults(data)
            break
        }
    }

    function rssAddFeed(url) { jsonCmd({ cmd: "RssAdd", url: url }, null) }
    function rssRemoveFeed(url) { jsonCmd({ cmd: "RssRemove", url: url }, null) }
    function rssRefresh() { jsonCmd({ cmd: "RssRefresh" }, null) }
    function wallhavenSearch(query, tags, page) {
        jsonCmd({ cmd: "WallhavenSearch", q: query, tags: tags, page: page }, null)
    }
    function wallhavenRandom(count) { jsonCmd({ cmd: "WallhavenRandom", count: count }, null) }

    // ── VFS / Disk control ─────────────────────────────────────────────────
    function refreshDiskUsage() {
        jsonCmd({ cmd: "VfsDiskUsage" }, _handleDiskUsage)
    }

    function refreshRemovableDevices() {
        jsonCmd({ cmd: "DiskList" }, _handleRemovableDevices)
    }

    function mountDevice(device) { jsonCmd({ cmd: "DiskMount", device: device }, null) }
    function unmountDevice(device) { jsonCmd({ cmd: "DiskUnmount", device: device }, null) }
    function ejectDevice(device) { jsonCmd({ cmd: "DiskEject", device: device }, null) }

    function searchFiles(query, maxResults) {
        const max = maxResults || 50
        jsonCmd({ cmd: "VfsSearch", q: query, max_results: max }, root.fileSearchResults)
    }

    // ── Wallpaper control ─────────────────────────────────────────────────
    function _handleWallpaperEvent(data) {
        switch (data.event) {
        case "changed":
            wallpaperEvent(data)
            break
        case "backend_changed":
            _updateWallpaperBackends()
            break
        case "backend_not_available":
            wallpaperEvent(data)
            break
        case "error":
            wallpaperEvent(data)
            break
        }
    }

    function _updateWallpaperBackends() {
        jsonCmd({ cmd: "WallpaperBackends" }, (result) => {
            if (result && result.backends) {
                root.wallpaperBackends = result.backends
                for (let backend of result.backends) {
                    if (backend.name === "swww") {
                        root.wallpaperSwwwAvailable = backend.available
                        root.wallpaperSwwwRunning = backend.daemon_running
                    }
                }
            }
        })
    }

    function wallpaperStatus() {
        jsonCmd({ cmd: "WallpaperStatus" }, (result) => {
            if (result) {
                wallpaperEvent({ event: "status", data: result })
            }
        })
    }

    function wallpaperSet(path, monitor, transition) {
        const params = { path: path }
        if (monitor) params.monitor = monitor
        if (transition) params.transition = transition
        jsonCmd({ cmd: "WallpaperSet", ...params }, null)
    }

    function wallpaperGet(monitor) {
        jsonCmd({ cmd: "WallpaperGet", monitor: monitor }, (result) => {
            if (result) {
                wallpaperEvent({ event: "current", path: result.wallpaper })
            }
        })
    }

    function wallpaperQueryBackends() {
        _updateWallpaperBackends()
    }

    // ── System Info handlers ───────────────────────────────────────────────
    function _updateSystemInfo(data) {
        if (!data) return

        // Update compositor info
        if (data.compositor) {
            root.compositorName = data.compositor.name || ""
            root.compositorCapabilities = data.compositor.capabilities || {}

            // Convenience properties
            root.supportsWallpaperControl = data.compositor.capabilities?.wallpaper_control || false
            root.supportsBlur = data.compositor.capabilities?.blur || false
            root.supportsLayerShell = data.compositor.capabilities?.layer_shell || false
        }

        // Could also store OS info, hardware, etc. if needed
        if (data.os) {
            root.osInfo = {
                name: data.os.name,
                kernel: data.os.kernel,
                prettyName: data.os.pretty_name
            }
        }
    }

    function querySystemInfo() {
        jsonCmd({ cmd: "SystemInfo" }, _updateSystemInfo)
    }
}
