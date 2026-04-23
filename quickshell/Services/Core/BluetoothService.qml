pragma Singleton

import QtQuick
import Quickshell

import qs.Common
import qs.Services.UI

Singleton {
  id: root

  readonly property bool bluetoothAvailable: true
  readonly property bool enabled: CrawlDSService.optBtPowered

  readonly property bool scanningActive: CrawlDSService.btDiscovering
  property bool discoverable: false

  readonly property var devices: CrawlDSService.btDevices || []

  readonly property var devicesList: {
    var d = devices
    if (!d) return []
    if (Array.isArray(d)) return d
    if (typeof d.values === "function") return d.values()
    return []
  }

  readonly property var connectedDevices: {
    var devs = devicesList
    if (!devs || !devs.length) return []
    return devs.filter(function(d) { return d && d.connected })
  }

  //property bool airplaneModeEnabled: Settings.data.network.airplaneModeEnabled
  //property bool airplaneModeToggled: false
  property bool ctlAvailable: false

  property bool pinRequired: false

  function init() {
    Logger.i("Bluetooth", "Service started - using CrawlDS core")
  }

  // Component.onCompleted: {
  //   init()
  //   if (root.airplaneModeEnabled) {
  //     Quickshell.execDetached(["rfkill", "block", "bluetooth"])
  //   }
  // }

  // function setAirplaneMode(state) {
  //   if (state) {
  //     Quickshell.execDetached(["rfkill", "block", "bluetooth"])
  //   } else {
  //     Quickshell.execDetached(["rfkill", "unblock", "bluetooth"])
  //   }
  //   root.airplaneModeToggled = true
  //   Settings.data.network.airplaneModeEnabled = state
  //   ToastService.showNotice("Airplane Mode", state ? "Enabled" : "Disabled", state ? "plane" : "plane-off")
  //   Logger.i("AirplaneMode", state ? "Bluetooth adapter blocked" : "Bluetooth adapter unblocked")
  //   root.airplaneModeToggled = false
  // }

  function setBluetoothEnabled(state) {
    Logger.i("Bluetooth", "setBluetoothEnabled", state)
    CrawlDSService.setBtPowered(state)
  }

  function setScanActive(active) {
    Logger.i("Bluetooth", "setScanActive", active)
    if (active) {
      CrawlDSService.startBtScan()
    } else {
      CrawlDSService.setBtDiscoverable(false)
    }
  }

  function setDiscoverable(state) {
    Logger.i("Bluetooth", "setDiscoverable", state)
    CrawlDSService.setBtDiscoverable(state)
  }

  function sortDevices(devList) {
    if (!devList || !Array.isArray(devList)) return []
    return devList.sort(function(a, b) {
      var aName = a.name || a.deviceName || ""
      var bName = b.name || b.deviceName || ""

      var aReal = aName.indexOf(" ") !== -1 && aName.length > 3
      var bReal = bName.indexOf(" ") !== -1 && bName.length > 3

      if (aReal && !bReal) return -1
      if (!aReal && bReal) return  1

      var aSig = (a.rssi !== undefined && a.rssi > 0) ? a.rssi : 0
      var bSig = (b.rssi !== undefined && b.rssi > 0) ? b.rssi : 0
      return bSig - aSig
    })
  }

  function dedupeDevices(list) {
    if (!list || !Array.isArray(list)) return []
    var seen = {}
    var result = []
    for (var i = 0; i < list.length; i++) {
      var d = list[i]
      if (!d) continue
      var key = deviceKey(d)
      if (!key || seen[key]) continue
      seen[key] = true
      result.push(d)
    }
    return result
  }

  function macFromDevice(device) {
    if (!device) return ""
    return String(device.address || device.bdaddr || device.mac || "").trim()
  }

  function deviceKey(device) {
    if (!device) return ""
    return macFromDevice(device) || (device.name || device.deviceName || "")
  }

  function getDeviceIcon(device) {
    if (!device) return "bt-device-generic"
    var name = device.name || device.deviceName || ""
    var icon = device.icon || ""
    var normalized = icon.toLowerCase().trim()

    var iconMap = {
      "audio-card":        "headphones",
      "audio-headset":     "headset",
      "audio-headphones":  "headphones",
      "speaker":           "speaker",
      "input-keyboard":    "keyboard",
      "input-mouse":       "mouse",
      "input-gaming":      "gamepad",
      "phone":             "smartphone",
      "watch":              "watch",
    }

    if (normalized && iconMap[normalized]) return iconMap[normalized]

    var n = name.toLowerCase()
    if (n.indexOf("headphone") !== -1 || n.indexOf("airpod") !== -1 || n.indexOf("earbud") !== -1 || n.indexOf("buds") !== -1) return "headphones"
    if (n.indexOf("speaker") !== -1 || n.indexOf("soundbar") !== -1) return "speaker"
    if (n.indexOf("keyboard") !== -1) return "keyboard"
    if (n.indexOf("mouse") !== -1) return "mouse"
    if (n.indexOf("gamepad") !== -1 || n.indexOf("controller") !== -1) return "gamepad"
    if (n.indexOf("camera") !== -1) return "camera"
    if (n.indexOf("phone") !== -1 || n.indexOf("iphone") !== -1 || n.indexOf("android") !== -1) return "smartphone"
    if (n.indexOf("watch") !== -1 || n.indexOf("fitbit") !== -1) return "watch"
    if (n.indexOf("printer") !== -1) return "printer"
    return "bt-device-generic"
  }

  function canConnect(device) {
    if (!device) return false
    return !device.connected && device.paired
  }

  function canDisconnect(device) {
    if (!device) return false
    return device.connected
  }

  function canPair(device) {
    if (!device) return false
    return !device.connected && !device.paired
  }

  function isDeviceBusy(device) {
    return false
  }

  function getSignalPercent(device) {
    if (!device) return null
    var rssi = device.rssi
    if (rssi === undefined || rssi === null || rssi === 0) return null

    var min = -100, max = -30
    var pct = Math.round(((rssi - min) / (max - min)) * 100)
    return Math.max(0, Math.min(100, pct))
  }

  function getSignalIcon(device) {
    var p = getSignalPercent(device)
    if (p === null || p === undefined) return "signal-off"
    if (p >= 80) return "signal-high"
    if (p >= 55) return "signal-medium"
    if (p >= 30) return "signal-low"
    return "signal-poor"
  }

  function getSignalStrength(device) {
    var p = getSignalPercent(device)
    if (p === null) return "Signal: Unknown"
    if (p >= 80) return "Signal: Excellent"
    if (p >= 60) return "Signal: Good"
    if (p >= 40) return "Signal: Fair"
    if (p >= 20) return "Signal: Poor"
    return "Signal: Very poor"
  }

  function getBatteryPercent(device) {
    if (!device) return null
    return device.battery || null
  }

  function batteryAvailable(device) {
    if (!device) return false
    return getBatteryPercent(device) !== null
  }

  function getStatusKey(device) {
    return ""
  }

  function connectDeviceWithTrust(device) {
    if (!device) return
    var addr = macFromDevice(device)
    CrawlDSService.setBtTrusted(addr, true)
    CrawlDSService.connectBtDevice(addr)
  }

  function disconnectDevice(device) {
    if (!device) return
    CrawlDSService.disconnectBtDevice(macFromDevice(device))
  }

  function forgetDevice(device) {
    if (!device) return
    CrawlDSService.forgetBtDevice(macFromDevice(device))
  }

  function unpairDevice(device) {
    forgetDevice(device)
  }

  function getDeviceAutoConnect(device) {
    return false
  }

  function setDeviceAutoConnect(device, enabled) {
    if (!device) return
    var addr = macFromDevice(device)
    CrawlDSService.setBtTrusted(addr, enabled)
  }

  function pairDevice(device) {
    if (!device) return
    CrawlDSService.pairBtDevice(macFromDevice(device))
    ToastService.showNotice("Bluetooth", "Pairing...", "bluetooth")
  }

  function submitPin(pin) {
    ToastService.showWarning("Bluetooth", "PIN pairing not supported - use backend")
  }

  function cancelPairing() {
    pinRequired = false
  }
}