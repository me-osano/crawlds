pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.Common

Singleton {
    id: root

    signal brightnessChanged(bool showOsd)
    signal monitorsListChanged()
    signal ddcMonitorsListChanged()
    signal monitorBrightnessChanged(var monitor, real newBrightness)
    signal deviceBrightnessUpdated(string deviceId, real brightness)

    property var devices: []
    property var deviceBrightness: ({})
    property var deviceBrightnessUserSet: ({})
    property var deviceMaxCache: ({})
    property var userControlledDevices: ({})

    property string currentDevice: ""
    property int brightnessVersion: 0

    property bool brightnessAvailable: devices.length > 0
    property bool suppressOsd: true

    property list<Monitor> monitors: variants.instances

    property list<var> ddcMonitors: []
    readonly property list<Monitor> allMonitors: monitors
    property bool appleDisplayPresent: false
    property list<var> availableBacklightDevices: []

    property int brightnessLevel: {
        brightnessVersion
        const deviceToUse = currentDevice || getDefaultDevice()
        if (!deviceToUse) return 50
        return getDeviceBrightness(deviceToUse)
    }

    Variants {
        id: variants
        model: Quickshell.screens
        Monitor {}
    }

    function init() {
        Logger.i("BrightnessService", "Service started")
        _loadPersistedState()
        scanBacklightDevices()
        if (Settings.data.brightness.enableDdcSupport) {
            ddcProc.running = true
        }
    }

    function _loadPersistedState() {
        var saved = Settings.data.brightness?.deviceBrightness
        if (saved && typeof saved === "object") {
            deviceBrightness = saved
        }
        var savedUserSet = Settings.data.brightness?.deviceBrightnessUserSet
        if (savedUserSet && typeof savedUserSet === "object") {
            deviceBrightnessUserSet = savedUserSet
        }
        var savedDevice = Settings.data.brightness?.lastDevice
        if (savedDevice) {
            currentDevice = savedDevice
        }
    }

    function _savePersistedState() {
        var newSettings = Object.assign({}, Settings.data.brightness || {})
        newSettings.deviceBrightness = deviceBrightness
        newSettings.deviceBrightnessUserSet = deviceBrightnessUserSet
        newSettings.lastDevice = currentDevice
        Settings.data.brightness = newSettings
    }

    function markDeviceUserControlled(deviceId) {
        var newControlled = Object.assign({}, userControlledDevices)
        newControlled[deviceId] = Date.now()
        userControlledDevices = newControlled
    }

    function isDeviceUserControlled(deviceId) {
        var controlTime = userControlledDevices[deviceId]
        if (!controlTime) return false
        return (Date.now() - controlTime) < 1000
    }

    function getDefaultDevice() {
        for (var i = 0; i < devices.length; i++) {
            if (devices[i].class === "backlight") {
                return devices[i].id
            }
        }
        return devices.length > 0 ? devices[0].id : ""
    }

    function getDeviceBrightness(deviceId) {
        if (!deviceId) return 50
        if (deviceBrightness[deviceId] !== undefined) {
            return deviceBrightness[deviceId]
        }
        return 50
    }

    function setDeviceBrightness(deviceId, brightness) {
        var newBrightness = Object.assign({}, deviceBrightness)
        newBrightness[deviceId] = brightness
        deviceBrightness = newBrightness
        brightnessVersion++
        _savePersistedState()
        deviceBrightnessUpdated(deviceId, brightness)
    }

    function setCurrentDevice(deviceId) {
        if (currentDevice === deviceId) return
        currentDevice = deviceId
        _savePersistedState()
    }

    function getMonitorForScreen(screen) {
        return monitors.find(m => m.modelData === screen)
    }

    function getAvailableMethods() {
        var methods = []
        if (Settings.data.brightness.enableDdcSupport && monitors.some(m => m.isDdc)) {
            methods.push("ddcutil")
        }
        if (monitors.some(m => !m.isDdc)) {
            methods.push("internal")
        }
        if (appleDisplayPresent) {
            methods.push("apple")
        }
        return methods
    }

    function _clamp01(v) {
        if (v === undefined || v === null || isNaN(v)) return 0
        return Math.max(0, Math.min(1, v))
    }

    function _step01() {
        var stepPct = Settings.data.brightness?.brightnessStep
        if (stepPct === undefined || stepPct === null || isNaN(stepPct)) stepPct = 5
        return Math.max(0.01, Math.min(1.0, Number(stepPct) / 100.0))
    }

    function _applyEnforcedMinimum(v01) {
        var clamped = _clamp01(v01)
        var enforceMin = Settings.data.brightness?.enforceMinimum !== false
        if (!enforceMin) return clamped
        if (clamped <= 0) return 0
        return Math.max(0.01, clamped)
    }

    function setBrightness(v01) {
        monitors.forEach(function(m) { m.setBrightnessDebounced(v01) })
    }

    function increaseBrightness() {
        monitors.forEach(function(m) { m.increaseBrightness() })
    }

    function decreaseBrightness() {
        monitors.forEach(function(m) { m.decreaseBrightness() })
    }

    function normalizeBacklightDevicePath(devicePath) {
        if (devicePath === undefined || devicePath === null) return ""
        var normalized = String(devicePath).trim()
        if (normalized === "") return ""
        if (normalized.startsWith("/sys/class/backlight/")) return normalized
        if (normalized.indexOf("/") === -1) return "/sys/class/backlight/" + normalized
        return normalized
    }

    function getBacklightDeviceName(devicePath) {
        var normalized = normalizeBacklightDevicePath(devicePath)
        if (normalized === "") return ""
        var parts = normalized.split("/")
        while (parts.length > 0 && parts[parts.length - 1] === "") parts.pop()
        return parts.length > 0 ? parts[parts.length - 1] : ""
    }

    function getMappedBacklightDevice(outputName) {
        var normalizedOutput = String(outputName || "").trim()
        if (normalizedOutput === "") return ""
        var mappings = Settings.data.brightness?.backlightDeviceMappings || []
        for (var i = 0; i < mappings.length; i++) {
            var mapping = mappings[i]
            if (!mapping || typeof mapping !== "object") continue
            if (String(mapping.output || "").trim() === normalizedOutput)
                return normalizeBacklightDevicePath(mapping.device || "")
        }
        return ""
    }

    function setMappedBacklightDevice(outputName, devicePath) {
        var normalizedOutput = String(outputName || "").trim()
        if (normalizedOutput === "") return
        var normalizedDevicePath = normalizeBacklightDevicePath(devicePath)
        var mappings = Settings.data.brightness?.backlightDeviceMappings || []
        var nextMappings = []
        var replaced = false
        for (var i = 0; i < mappings.length; i++) {
            var mapping = mappings[i]
            if (!mapping || typeof mapping !== "object") continue
            var mappingOutput = String(mapping.output || "").trim()
            var mappingDevice = normalizeBacklightDevicePath(mapping.device || "")
            if (mappingOutput === "" || mappingDevice === "") continue
            if (mappingOutput === normalizedOutput) {
                if (!replaced && normalizedDevicePath !== "") {
                    nextMappings.push({ "output": normalizedOutput, "device": normalizedDevicePath })
                }
                replaced = true
            } else {
                nextMappings.push({ "output": mappingOutput, "device": mappingDevice })
            }
        }
        if (!replaced && normalizedDevicePath !== "") {
            nextMappings.push({ "output": normalizedOutput, "device": normalizedDevicePath })
        }
        Settings.data.brightness.backlightDeviceMappings = nextMappings
    }

    function scanBacklightDevices() {
        if (!scanBacklightProc.running) scanBacklightProc.running = true
    }

    Connections {
        target: Settings.data.brightness
        function onEnableDdcSupportChanged() {
            if (Settings.data.brightness.enableDdcSupport) {
                ddcMonitors = []
                ddcProc.running = true
            } else {
                ddcMonitors = []
            }
        }
        function onBacklightDeviceMappingsChanged() {
            scanBacklightDevices()
            monitors.forEach(function(m) {
                if (m && !m.isDdc && !m.isAppleDisplay) m.initBrightness()
            })
        }
    }

    Connections {
        target: CrawlDSService
        function onBrightnessPercentChanged() {
            var pct = CrawlDSService.brightnessPercent
            var deviceId = CrawlDSService.brightnessDevice || getDefaultDevice()
            if (deviceId && !isDeviceUserControlled(deviceId)) {
                setDeviceBrightness(deviceId, pct)
            }
            monitors.forEach(function(m) {
                if (m && !m.isDdc && !m.isAppleDisplay) {
                    m.brightnessUpdated(m.brightness)
                }
            })
        }
    }

    Timer {
        id: osdSuppressTimer
        interval: 2000
        running: true
        onTriggered: suppressOsd = false
    }

    Process {
        id: scanBacklightProc
        command: ["sh", "-c", "for dev in /sys/class/backlight/*; do if [ -f \"$dev/brightness\" ] && [ -f \"$dev/max_brightness\" ]; then echo \"$dev\"; fi; done"]
        stdout: StdioCollector {
            onStreamFinished: {
                var data = text.trim()
                if (data === "") {
                    root.availableBacklightDevices = []
                    return
                }
                var lines = data.split("\n")
                var found = []
                var seen = {}
                for (var i = 0; i < lines.length; i++) {
                    var path = root.normalizeBacklightDevicePath(lines[i])
                    if (path === "" || seen[path]) continue
                    seen[path] = true
                    found.push(path)
                }
                root.availableBacklightDevices = found
                _updateDeviceList()
            }
        }
    }

    Process {
        id: ddcProc
        command: ["ddcutil", "detect", "--sleep-multiplier=0.5"]
        stdout: StdioCollector {
            onStreamFinished: {
                var displays = text.trim().split("\n\n")
                var detected = displays.map(function(d) {
                    var ddcModelMatch = d.match(/(This monitor does not support DDC\/CI|Invalid display)/)
                    var modelMatch = d.match(/Model:\s*(.*)/)
                    var busMatch = d.match(/I2C bus:[ ]*\/dev\/i2c-([0-9]+)/)
                    var connectorMatch = d.match(/DRM[_ ]connector:\s*card\d+-(.+)/)
                    var ddcModel = ddcModelMatch ? ddcModelMatch.length > 0 : false
                    var model = modelMatch ? modelMatch[1] : "Unknown"
                    var bus = busMatch ? busMatch[1] : "Unknown"
                    var connector = connectorMatch ? connectorMatch[1].trim() : ""
                    Logger.i("Brightness", "Detected DDC Monitor:", model, "connector:", connector, "bus:", bus, "isDdc:", !ddcModel)
                    return {
                        "model": model,
                        "busNum": bus,
                        "connector": connector,
                        "isDdc": !ddcModel
                    }
                })
                root.ddcMonitors = detected.filter(function(m) { return m.isDdc })
                ddcMonitorsListChanged()
            }
        }
    }

    Process {
        id: appleDisplayProc
        running: true
        command: ["sh", "-c", "which asdbctl >/dev/null 2>&1 && asdbctl get || echo ''"]
        stdout: StdioCollector {
            onStreamFinished: root.appleDisplayPresent = text.trim().length > 0
        }
    }

    function _updateDeviceList() {
        var newDevices = []
        for (var i = 0; i < availableBacklightDevices.length; i++) {
            var path = availableBacklightDevices[i]
            var name = getBacklightDeviceName(path)
            newDevices.push({
                "id": name,
                "path": path,
                "class": "backlight"
            })
        }
        for (var j = 0; j < ddcMonitors.length; j++) {
            var ddc = ddcMonitors[j]
            newDevices.push({
                "id": ddc.connector,
                "class": "ddc",
                "busNum": ddc.busNum,
                "model": ddc.model
            })
        }
        devices = newDevices
        if (devices.length > 0 && !currentDevice) {
            currentDevice = getDefaultDevice()
        }
        monitorsListChanged()
    }

    component Monitor: QtObject {
        id: monitor

        required property ShellScreen modelData

        readonly property bool isDdc: Settings.data.brightness.enableDdcSupport && root.ddcMonitors.some(function(m) { return m.connector === modelData.name })
        readonly property string busNum: root.ddcMonitors.find(function(m) { return m.connector === modelData.name })?.busNum ?? ""
        readonly property bool isAppleDisplay: root.appleDisplayPresent && String(modelData.model || "").startsWith("StudioDisplay")
        readonly property string method: isAppleDisplay ? "apple" : (isDdc ? "ddcutil" : "internal")

        readonly property bool brightnessControlAvailable: {
            if (isAppleDisplay) return true
            if (isDdc) return true
            return CrawlDSService.connected || (root.availableBacklightDevices.length > 0)
        }

        property real brightness: {
            if (isAppleDisplay || isDdc) return _readBrightnessFromSystem()
            var deviceId = root.currentDevice || root.getDefaultDevice()
            if (!deviceId) return 0
            var pct = root.getDeviceBrightness(deviceId)
            if (pct === undefined || pct === null || isNaN(pct)) return 0
            return root._clamp01(Number(pct) / 100.0)
        }
        property real lastBrightness: 0
        property real queuedBrightness: NaN
        property bool commandRunning: false

        property string backlightDevice: ""
        property string brightnessPath: ""
        property string maxBrightnessPath: ""
        property int maxBrightness: 100
        property bool ignoreNextChange: false
        property bool initInProgress: false

        signal brightnessUpdated(real newBrightness)

        property QtObject initProc: Process {
            stdout: StdioCollector {
                onStreamFinished: {
                    var dataText = text.trim()
                    if (dataText === "") return
                    if (monitor.isAppleDisplay) {
                        var val = parseInt(dataText)
                        if (!isNaN(val)) {
                            monitor.brightness = val / 101
                            monitor.lastBrightness = monitor.brightness
                            var deviceId = root.currentDevice || root.getDefaultDevice()
                            if (deviceId) {
                                root.setDeviceBrightness(deviceId, val)
                            }
                        }
                    } else if (monitor.isDdc) {
                        var parts = dataText.split(" ")
                        if (parts.length >= 4) {
                            var current = parseInt(parts[3])
                            var max = parseInt(parts[4])
                            if (!isNaN(current) && !isNaN(max) && max > 0) {
                                monitor.maxBrightness = max
                                monitor.brightness = current / max
                                monitor.lastBrightness = monitor.brightness
                                var deviceId = root.currentDevice || root.getDefaultDevice()
                                if (deviceId) {
                                    root.setDeviceBrightness(deviceId, Math.round((current / max) * 100))
                                }
                            }
                        }
                    } else {
                        var lines = dataText.split("\n")
                        if (lines.length >= 3) {
                            monitor.backlightDevice = lines[0]
                            monitor.brightnessPath = monitor.backlightDevice + "/brightness"
                            monitor.maxBrightnessPath = monitor.backlightDevice + "/max_brightness"
                            var current = parseInt(lines[1])
                            var max = parseInt(lines[2])
                            if (!isNaN(current) && !isNaN(max) && max > 0) {
                                monitor.maxBrightness = max
                                monitor.brightness = current / max
                                monitor.lastBrightness = monitor.brightness
                                var deviceId = root.getBacklightDeviceName(lines[0])
                                if (deviceId) {
                                    root.setDeviceBrightness(deviceId, Math.round((current / max) * 100))
                                }
                            }
                        }
                    }
                    monitor.initInProgress = false
                }
            }
            onExited: monitor.initInProgress = false
        }

        function _readBrightnessFromSystem() {
            var deviceId = root.currentDevice || root.getDefaultDevice()
            return root.getDeviceBrightness(deviceId) / 100.0
        }

        function _clamp01(v) {
            if (v === undefined || v === null || isNaN(v)) return 0
            return Math.max(0, Math.min(1, v))
        }

        function _step01() {
            var stepPct = Settings.data.brightness?.brightnessStep
            if (stepPct === undefined || stepPct === null || isNaN(stepPct)) stepPct = 5
            return Math.max(0.01, Math.min(1.0, Number(stepPct) / 100.0))
        }

        function _applyEnforcedMinimum(v01) {
            var clamped = _clamp01(v01)
            var enforceMin = Settings.data.brightness?.enforceMinimum !== false
            if (!enforceMin) return clamped
            if (clamped <= 0) return 0
            return Math.max(0.01, clamped)
        }

        function setBrightnessDebounced(value) {
            monitor.queuedBrightness = value
            debounceTimer.start()
        }

        function increaseBrightness() {
            var value = !isNaN(monitor.queuedBrightness) ? monitor.queuedBrightness : monitor.brightness
            var minVal = Settings.data.brightness?.enforceMinimum ? 0.01 : 0.0
            if (Settings.data.brightness?.enforceMinimum && value < minVal) {
                setBrightnessDebounced(Math.max(_step01(), minVal))
            } else {
                setBrightnessDebounced(value + _step01())
            }
        }

        function decreaseBrightness() {
            var value = !isNaN(monitor.queuedBrightness) ? monitor.queuedBrightness : monitor.brightness
            setBrightnessDebounced(value - _step01())
        }

        function setBrightness(value) {
            var minVal = Settings.data.brightness?.enforceMinimum ? 0.01 : 0.0
            value = Math.max(minVal, Math.min(1, value))
            var rounded = Math.round(value * 100)

            monitor.brightness = value
            monitor.lastBrightness = value

            var deviceId = root.currentDevice || root.getDefaultDevice()
            if (deviceId) {
                root.setDeviceBrightness(deviceId, rounded)
                root.markDeviceUserControlled(deviceId)
            }

            brightnessUpdated(value)
            root.brightnessChanged(!root.suppressOsd)
            root.monitorBrightnessChanged(monitor, value)

            if (isAppleDisplay) {
                monitor.commandRunning = true
                monitor.ignoreNextChange = true
                setBrightnessProc.command = ["asdbctl", "set", rounded]
                setBrightnessProc.running = true
            } else if (isDdc && busNum !== "") {
                monitor.commandRunning = true
                monitor.ignoreNextChange = true
                var ddcValue = Math.round(value * monitor.maxBrightness)
                setBrightnessProc.command = ["ddcutil", "-b", busNum, "--noverify", "--async", "--sleep-multiplier=0.05", "setvcp", "10", ddcValue]
                setBrightnessProc.running = true
            } else {
                CrawlDSService.setBrightness(_applyEnforcedMinimum(value) * 100)
            }
        }

        function refreshBrightnessFromSystem() {
            if (isAppleDisplay) {
                initProc.command = ["asdbctl", "get"]
                initProc.running = true
            } else if (isDdc && busNum !== "") {
                initProc.command = ["ddcutil", "-b", busNum, "--sleep-multiplier=0.05", "getvcp", "10", "--brief"]
                initProc.running = true
            } else if (!isDdc) {
                var preferred = root.getMappedBacklightDevice(modelData.name)
                var script = ["preferred=$1", "if [ -n \"$preferred\" ] && [ ! -d \"$preferred\" ]; then preferred=/sys/class/backlight/$preferred; fi", "selected=\"\"",
                    "if [ -n \"$preferred\" ] && [ -f \"$preferred/brightness\" ] && [ -f \"$preferred/max_brightness\" ]; then selected=\"$preferred\"; else for dev in /sys/class/backlight/*; do if [ -f \"$dev/brightness\" ] && [ -f \"$dev/max_brightness\" ]; then selected=\"$dev\"; break; fi; done; fi",
                    "if [ -n \"$selected\" ]; then echo \"$selected\"; cat \"$selected/brightness\"; cat \"$selected/max_brightness\"; fi"].join("; ")
                initProc.command = ["sh", "-c", script, "sh", preferred]
                initProc.running = true
            }
        }

        function initBrightness() {
            if (isDdc || isAppleDisplay) {
                refreshBrightnessFromSystem()
            }
        }

        property QtObject setBrightnessProc: Process {
            stdout: StdioCollector {}
            onExited: {
                monitor.commandRunning = false
                if (!isNaN(monitor.queuedBrightness)) {
                    Qt.callLater(function() {
                        monitor.setBrightness(monitor.queuedBrightness)
                        monitor.queuedBrightness = NaN
                    })
                }
            }
        }

        property QtObject debounceTimer: Timer {
            interval: monitor.isDdc ? 250 : 33
            onTriggered: {
                if (!isNaN(monitor.queuedBrightness)) {
                    monitor.setBrightness(monitor.queuedBrightness)
                    monitor.queuedBrightness = NaN
                }
            }
        }

        onBusNumChanged: initBrightness()
        onIsDdcChanged: initBrightness()
        Component.onCompleted: initBrightness()
    }

    reloadableId: "brightness"

    Component.onCompleted: {
        Logger.i("Brightness", "Service started")
        scanBacklightDevices()
        if (Settings.data.brightness.enableDdcSupport) {
            ddcProc.running = true
        }
    }

    onMonitorsChanged: {
        ddcMonitors = []
        scanBacklightDevices()
        if (Settings.data.brightness.enableDdcSupport) {
            ddcProc.running = true
        }
    }
}