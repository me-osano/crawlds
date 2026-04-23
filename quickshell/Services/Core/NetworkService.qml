pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

import qs.Common

Singleton {
    id: root

    property var networks: ({})
    property bool scanning: false
    property bool scanningActive: false
    property string connectingTo: ""
    property bool connecting: false
    property string disconnectingFrom: ""
    property string forgettingNetwork: ""
    property string lastError: ""

    property bool wifiAvailable: false
    property bool ethernetConnected: false
    property var ethernetInterfaces: ([])
    property string activeWifiIf: ""
    property string activeEthernetIf: ""
    property var activeWifiDetails: ({})
    property var activeEthernetDetails: ({})
    property double activeWifiDetailsTimestamp: 0
    property double activeEthernetDetailsTimestamp: 0
    property bool hotspotActive: false
    property var hotspotStatus: ({})
    property bool hotspotStarting: false
    property bool hotspotStopping: false
    property int activeWifiDetailsTtlMs: 5000
    property int activeEthernetDetailsTtlMs: 5000
    property bool detailsLoading: false
    property bool ethernetDetailsLoading: false
    property bool ignoreScanResults: false
    property bool scanPending: false

    readonly property bool wifiEnabled: CrawlDSService.netWifiEnabled
    readonly property string networkConnectivity: CrawlDSService.netConnectivity
    readonly property bool internetConnectivity: CrawlDSService.netConnectivity === "full"
    readonly property string activeSsid: CrawlDSService.netActiveSsid
    readonly property bool wifiAvailableFromDaemon: CrawlDSService.netWifiAvailable
    readonly property bool ethernetAvailableFromDaemon: CrawlDSService.netEthernetAvailable

    property bool airplaneModeEnabled: Settings.data.network.airplaneModeEnabled
    property string _lastConnectivity: "unknown"
    property bool _internetCheckInProgress: false

    readonly property var supportedSecurityTypes: ([
        { key: "open", name: "Open" },
        { key: "wpa-psk", name: "WPA/WPA2 Personal" },
        { key: "wpa2-psk", name: "WPA2 Personal" },
        { key: "wpa3-psk", name: "WPA3 Personal" },
        { key: "wpa-eap", name: "WPA/WPA2 Enterprise" },
        { key: "wpa2-eap", name: "WPA2 Enterprise" },
        { key: "wpa3-eap", name: "WPA3 Enterprise" }
    ])

    function init() {
        Logger.i("Network", "Service started - using CrawlDS backend")
        refreshNetworks()
        refreshEthernet()
        refreshActiveWifiDetails()
        refreshHotspotStatus()
        wifiAvailable = wifiAvailableFromDaemon
    }

    Component.onCompleted: {
        init()
        restoreCache()
    }

    function hasEthernet() {
        return ethernetInterfaces && ethernetInterfaces.length > 0
    }

    function setWifiEnabled(on) {
        Logger.i("Network", "setWifiEnabled", on)
        Settings.data.network.wifiEnabled = on
        CrawlDSService.jsonCmd({ cmd: "WifiPower", on: on }, null)
        if (on) {
            scan()
            refreshActiveWifiDetails()
        }
    }

    function checkInternetConnectivity() {
        const current = networkConnectivity
        const wasLimited = _lastConnectivity === "limited" || _lastConnectivity === "portal"
        const isNowFull = current === "full"

        if (wasLimited && isNowFull) {
            _lastConnectivity = current
            return
        }

        if ((current === "limited" || current === "portal") && _lastConnectivity !== current) {
            _lastConnectivity = current
            if (!_internetCheckInProgress) {
                _internetCheckInProgress = true
                ConnectivityCheckProcess.running = true
            }
        } else if (current === "full" && _lastConnectivity !== "full") {
            _lastConnectivity = current
        }
    }

    Process {
        id: connectivityCheckProcess
        running: false
        command: ["sh", "-c", "curl -fsI --max-time 5 https://cloudflare.com/cdn-cgi/trace >/dev/null 2>&1 || curl -fsI --max-time 5 https://www.google.com >/dev/null 2>&1"]
        onExited: {
            _internetCheckInProgress = false
            if (exitCode !== 0) {
                activeSsid = activeSsid || root.cachedLastConnected
                if (activeSsid && internetConnectivity !== root._lastConnectivity) {
                    ToastService.showWarning(activeSsid, "Connected without internet")
                }
            }
            root._lastConnectivity = networkConnectivity
        }
    }

    function setAirplaneMode(state) {
        Logger.i("Network", "setAirplaneMode", state)
        Settings.data.network.airplaneModeEnabled = state
        // if (typeof BluetoothService !== "undefined" && BluetoothService.setAirplaneMode) {
        //     BluetoothService.setAirplaneMode(state)
        // }
        if (state) {
            setWifiEnabled(false)
        }
        ToastService.showNotice("Airplane Mode", state ? "Enabled" : "Disabled", state ? "plane" : "plane-off")
    }

    function scan() {
        if (scanning) {
            ignoreScanResults = true
            scanPending = true
            return
        }
        scanning = true
        scanningActive = true
        ignoreScanResults = false
        CrawlDSService.jsonCmd({ cmd: "WifiScan" }, null)
        refreshNetworks()
    }

    function refreshNetworks() {
        CrawlDSService.jsonCmd({ cmd: "WifiList" }, function (list) {
            if (ignoreScanResults) {
                scanning = false
                if (scanPending) {
                    scanPending = false
                    scanDelayTimer.restart()
                }
                return
            }
            applyWifiList(list)
            scanning = false
        })
    }

    function refreshActiveWifiDetails() {
        const now = Date.now()
        if (detailsLoading) {
            return
        }
        if (activeWifiDetails && activeWifiIf && (now - activeWifiDetailsTimestamp) < activeWifiDetailsTtlMs) {
            return
        }
        detailsLoading = true
        CrawlDSService.jsonCmd({ cmd: "WifiDetails" }, function (details) {
            activeWifiDetails = details || ({})
            activeWifiIf = details?.ifname || ""
            activeWifiDetailsTimestamp = Date.now()
            detailsLoading = false
        })
    }

    function refreshEthernet() {
        CrawlDSService.jsonCmd({ cmd: "EthList" }, function (list) {
            ethernetInterfaces = Array.isArray(list) ? list : []
            ethernetConnected = ethernetInterfaces.some(i => i.connected)
            wifiAvailable = wifiAvailableFromDaemon
            if (!activeEthernetIf || activeEthernetIf.length === 0) {
                const active = ethernetInterfaces.find(i => i.connected)
                if (active) {
                    activeEthernetIf = active.ifname
                }
            }
        })
    }

    function refreshActiveEthernetDetails() {
        const now = Date.now()
        if (ethernetDetailsLoading) {
            return
        }
        if (!ethernetConnected) {
            activeEthernetDetails = ({})
            activeEthernetDetailsTimestamp = now
            return
        }
        if (activeEthernetDetails && activeEthernetIf && (now - activeEthernetDetailsTimestamp) < activeEthernetDetailsTtlMs) {
            return
        }
        ethernetDetailsLoading = true
        const iface = activeEthernetIf && activeEthernetIf.length > 0 ? activeEthernetIf : ""
        CrawlDSService.jsonCmd({ cmd: "EthDetails", interface: iface }, function (details) {
            activeEthernetDetails = details || ({})
            if (details && details.ifname) {
                activeEthernetIf = details.ifname
            }
            activeEthernetDetailsTimestamp = Date.now()
            ethernetDetailsLoading = false
        })
    }

    function connect(ssid, password, hidden, securityKey, identity, enterprise) {
        connecting = true
        connectingTo = ssid || ""
        const payload = {
            ssid: ssid,
            password: password
        }
        cacheAdapter.lastConnected = ssid
        cacheFileView.writeAdapter()
        CrawlDSService.jsonCmd({ cmd: "WifiConnect", ssid: ssid, password: password }, null)
        refreshNetworks()
    }

    function disconnect(ssid) {
        disconnectingFrom = ssid || ""
        CrawlDSService.jsonCmd({ cmd: "WifiDisconnect" }, null)
        refreshNetworks()
    }

    function forget(ssid) {
        forgettingNetwork = ssid || ""
        CrawlDSService.jsonCmd({ cmd: "WifiForget", ssid: ssid || "" }, null)
        Qt.callLater(function () {
            forgettingNetwork = ""
            let nets = networks
            if (nets[ssid]) {
                delete nets[ssid]
                networks = ({})
                networks = nets
            }
        })
    }

    function connectEthernet(iface) {
        CrawlDSService.jsonCmd({ cmd: "EthConnect", interface: iface }, null)
        refreshEthernet()
    }

    function disconnectEthernet(iface) {
        CrawlDSService.jsonCmd({ cmd: "EthDisconnect", interface: iface }, null)
        refreshEthernet()
    }

    function startHotspot(ssid, password, iface, band, channel, backend) {
        hotspotStarting = true
        const cmd = { cmd: "HotspotStart", ssid: ssid || "CrawlDS-Hotspot" }
        if (password && password.length > 0) cmd.password = password
        if (iface) cmd.iface = iface
        if (band) cmd.band = band
        if (channel) cmd.channel = channel
        if (backend && backend.length > 0) cmd.backend = backend
        CrawlDSService.jsonCmd(cmd, null)
        refreshHotspotStatus()
    }

    function stopHotspot() {
        hotspotStopping = true
        CrawlDSService.jsonCmd({ cmd: "HotspotStop" }, null)
        Qt.callLater(function () {
            hotspotStopping = false
            hotspotActive = false
            hotspotStatus = ({})
        })
    }

    function refreshHotspotStatus() {
        CrawlDSService.jsonCmd({ cmd: "HotspotStatus" }, function (status) {
            hotspotActive = status && status.active
            hotspotStatus = status || ({})
            hotspotStarting = false
            hotspotStopping = false
        })
    }

    function isSecured(security) {
        if (!security) return false
        return security !== "open"
    }

    function isEnterprise(security) {
        if (!security) return false
        return security.indexOf("eap") !== -1
    }

    function getSignalStrengthLabel(signal) {
        if (signal >= 80) return "Excellent"
        if (signal >= 60) return "Good"
        if (signal >= 40) return "Fair"
        if (signal >= 20) return "Weak"
        return "Poor"
    }

    function getSignalInfo(signal, connected) {
        return { icon: signalIcon(signal, connected), label: getSignalStrengthLabel(signal) }
    }

    function signalIcon(signal, connected) {
        if (!connected) return "wifi-off"
        if (signal >= 70) return "wifi-2"
        if (signal >= 35) return "wifi-1"
        return "wifi-0"
    }

    function applyWifiList(list) {
        if (!Array.isArray(list)) {
            networks = ({})
            restoreCache()
            return
        }
        const mapped = ({})
        for (let i = 0; i < list.length; i++) {
            const n = list[i]
            if (!n || !n.ssid) continue
            mapped[n.ssid] = {
                ssid: n.ssid,
                signal: n.signal || 0,
                secured: n.secured || false,
                connected: n.connected || false,
                existing: n.existing || false,
                cached: n.cached || false,
                passwordRequired: n.password_required || false,
                security: n.security || (n.secured ? "wpa2" : "open"),
                frequency: n.frequency_mhz || null,
                bssid: n.bssid || "",
                lastSeen: n.last_seen_ms || 0
            }
        }
        networks = mapped
        updateCache(mapped)
        if (Object.keys(mapped).length > 0) {
            scanningActive = false
        }
        if (connecting) {
            connecting = false
            connectingTo = ""
        }
        if (disconnectingFrom) {
            disconnectingFrom = ""
        }
    }

    function updateCache(mapped) {
        cacheAdapter.networks = mapped
        cacheSaveDebounce.restart()
    }

    function restoreCache() {
        if (cachedNetworks && Object.keys(cachedNetworks).length > 0) {
            networks = cachedNetworks
        }
    }

    Timer {
        id: scanDelayTimer
        interval: 200
        repeat: false
        onTriggered: scan()
    }

    Connections {
        target: CrawlDSService
        function onNetConnectivityChanged() {
            refreshEthernet()
            checkInternetConnectivity()
        }
        function onNetWifiListUpdated(networks) {
            applyWifiList(networks)
            scanning = false
        }
        function onNetWifiScanStarted() {
            scanning = true
            scanningActive = true
        }
        function onNetWifiScanFinished() {
            scanning = false
        }
        function onNetWifiDetailsUpdated(details) {
            activeWifiDetails = details || ({})
            activeWifiIf = details?.ifname || ""
            activeWifiDetailsTimestamp = Date.now()
            detailsLoading = false
        }
        function onNetEthernetListUpdated(interfaces) {
            ethernetInterfaces = Array.isArray(interfaces) ? interfaces : []
            ethernetConnected = ethernetInterfaces.some(i => i.connected)
            if (!activeEthernetIf || activeEthernetIf.length === 0) {
                const active = ethernetInterfaces.find(i => i.connected)
                if (active) {
                    activeEthernetIf = active.ifname
                }
            }
        }
        function onNetEthernetDetailsUpdated(details) {
            activeEthernetDetails = details || ({})
            if (details && details.ifname) {
                activeEthernetIf = details.ifname
            }
            activeEthernetDetailsTimestamp = Date.now()
            ethernetDetailsLoading = false
        }
        function onNetWifiEnabledChanged() {
            if (CrawlDSService.netWifiEnabled) {
                refreshNetworks()
                refreshActiveWifiDetails()
            } else {
                networks = ({})
                activeWifiDetails = ({})
                activeWifiIf = ""
            }
        }
        function onNetWifiAvailableChanged() {
            wifiAvailable = CrawlDSService.netWifiAvailable
        }
        function onNetHotspotStatusChanged(data) {
            hotspotActive = data && data.active
            hotspotStatus = data || ({})
            hotspotStarting = false
            hotspotStopping = false
        }
        function onNetHotspotStarted(status) {
            hotspotActive = status && status.active
            hotspotStatus = status || ({})
            hotspotStarting = false
            hotspotStopping = false
        }
        function onNetHotspotStopped() {
            hotspotActive = false
            hotspotStatus = ({})
            hotspotStarting = false
            hotspotStopping = false
        }
    }

    Connections {
        target: ShellState
        function onSessionResumedChanged() {
            refreshEthernet()
            refreshNetworks()
            refreshActiveWifiDetails()
            refreshActiveEthernetDetails()
            refreshHotspotStatus()
        }
    }

    property string cacheFile: Settings.cacheDir + "network.json"
    readonly property var cachedNetworks: cacheAdapter.networks
    readonly property string cachedLastConnected: cacheAdapter.lastConnected || ""

    FileView {
        id: cacheFileView
        path: root.cacheFile
        printErrors: false

        JsonAdapter {
            id: cacheAdapter
            property var networks: ({})
            property string lastConnected: ""
        }

        onLoadFailed: {
            cacheAdapter.networks = ({})
        }
    }

    Timer {
        id: cacheSaveDebounce
        interval: 1000
        repeat: false
        onTriggered: cacheFileView.writeAdapter()
    }
}