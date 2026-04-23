import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Modules.Panels.Network
import qs.Modules.Settings
import qs.Services.Core
import qs.Services.UI
import qs.Widgets

// Network card: inline expandable network panel for control center
NBox {
  id: root

  property ShellScreen screen
  property bool expanded: false
  
  // Expose the target height (non-animated) for parent layout calculations
  readonly property real targetHeight: expanded ? contentColumn.implicitHeight + Style.margin2M : 0

  // Password and expand states for WiFi networks
  property string passwordSsid: ""
  property string expandedSsid: ""
  
  // Tab selection: 0 = WiFi, 1 = Ethernet
  property int currentTab: 0

  // Connected WiFi SSID
  readonly property string connectedSsid: {
    if (!Settings.data.network.wifiEnabled)
      return "";
    for (const net in NetworkService.networks) {
      if (NetworkService.networks[net].connected) {
        return net;
      }
    }
    return "";
  }

  // Computed network lists
  readonly property var knownNetworks: {
    if (!Settings.data.network.wifiEnabled)
      return [];

    var nets = Object.values(NetworkService.networks);
    var known = nets.filter(n => n.connected || n.existing || n.cached);

    // Sort: connected first, then by signal strength
    known.sort((a, b) => {
      if (a.connected !== b.connected)
        return b.connected - a.connected;
      return b.signal - a.signal;
    });

    return known;
  }

  readonly property var availableNetworks: {
    if (!Settings.data.network.wifiEnabled)
      return [];

    var nets = Object.values(NetworkService.networks);
    var available = nets.filter(n => !n.connected && !n.existing && !n.cached);

    // Sort by signal strength
    available.sort((a, b) => b.signal - a.signal);

    return available;
  }

  clip: true
  // When used as overlay, visibility is controlled externally
  // When expanded is true, show the card content
  visible: expanded
  
  Behavior on implicitHeight {
    NumberAnimation {
      duration: Style.animationNormal
      easing.type: Easing.InOutQuad
    }
  }

  // For overlay mode, always show full height when expanded
  implicitHeight: expanded ? contentColumn.implicitHeight + Style.margin2M : 0

  // Trigger scan when expanded
  onExpandedChanged: {
    if (expanded) {
      if (Settings.data.network.wifiEnabled && !NetworkService.scanning) {
        NetworkService.scan();
      }
      NetworkService.refreshActiveWifiDetails();
    } else {
      // Reset states when collapsed
      passwordSsid = "";
      expandedSsid = "";
    }
  }

  ColumnLayout {
    id: contentColumn
    anchors.fill: parent
    anchors.margins: Style.marginM
    spacing: Style.marginM

    // Header row with network status
    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginS

      NIcon {
        id: headericon
        pointSize: Style.fontSizeL
        color: {
          try {
            if (NetworkService.ethernetConnected) {
              return Color.mPrimary;
            }
            for (const net in NetworkService.networks) {
              if (NetworkService.networks[net].connected) {
                return Color.mPrimary;
              }
            }
            return Color.mOnSurfaceVariant;
          } catch (e) {
            return Color.mOnSurfaceVariant;
          }
        }
        icon: {
          try {
            if (NetworkService.ethernetConnected) {
              return NetworkService.internetConnectivity ? "ethernet" : "ethernet-off";
            }
            let connected = false;
            let signalStrength = 0;
            for (const net in NetworkService.networks) {
              if (NetworkService.networks[net].connected) {
                connected = true;
                signalStrength = NetworkService.networks[net].signal;
                break;
              }
            }
            return connected ? NetworkService.signalIcon(signalStrength, true) : "wifi-off";
          } catch (error) {
            Logger.e("Network Panel", "Error getting icon:", error);
            return "wifi-off";
          }
        }
      }

      ColumnLayout {
        Layout.fillWidth: true
        spacing: 0

        NText {
          text: {
            try {
              if (NetworkService.ethernetConnected) {
                return "Ethernet";
              }
              for (const net in NetworkService.networks) {
                if (NetworkService.networks[net].connected) {
                  return net;
                }
              }
              return "Not Connected";
            } catch (e) {
              return "Network";
            }
          }
          font.weight: Style.fontWeightBold
          pointSize: Style.fontSizeS
        }

        NText {
          text: {
            try {
              if (NetworkService.ethernetConnected) {
                const d = NetworkService.activeEthernetDetails || ({});
                const speed = (d.speed && d.speed.length > 0) ? d.speed : "";
                return speed ? "Speed: " + speed : "Connected";
              }
              for (const net in NetworkService.networks) {
                if (NetworkService.networks[net].connected) {
                  const w = NetworkService.activeWifiDetails || ({});
                  const rate = (w.rateShort && w.rateShort.length > 0) ? w.rateShort : (w.rate || "");
                  return rate ? "Speed: " + rate : "Connected";
                }
              }
              return "Enable Wi-Fi to connect";
            } catch (e) {
              return "—";
            }
          }
          pointSize: Style.fontSizeXS
          color: Color.mOnSurfaceVariant
        }
      }

      Item {
        Layout.fillWidth: true
      }

      // Refresh button (top-right)
      NIconButton {
        visible: root.currentTab === 0
        icon: "refresh"
        baseSize: Style.baseWidgetSize * 0.8
        tooltipText: "Scan for networks"
        enabled: Settings.data.network.wifiEnabled && !NetworkService.scanning
        onClicked: NetworkService.scan()
      }

      // WiFi tab button
      NIconButton {
        icon: "wifi"
        baseSize: Style.baseWidgetSize * 0.8
        tooltipText: "Wi-Fi"
        colorBg: root.currentTab === 0 ? Color.mPrimary : Color.mSurfaceVariant
        colorFg: root.currentTab === 0 ? Color.mOnPrimary : Color.mOnSurfaceVariant
        onClicked: root.currentTab = 0
      }

      // Ethernet tab button
      NIconButton {
        icon: "ethernet"
        baseSize: Style.baseWidgetSize * 0.8
        tooltipText: "Ethernet"
        colorBg: root.currentTab === 1 ? Color.mPrimary : Color.mSurfaceVariant
        colorFg: root.currentTab === 1 ? Color.mOnPrimary : Color.mOnSurfaceVariant
        onClicked: root.currentTab = 1
      }

      // Settings button
      NIconButton {
        icon: "settings"
        baseSize: Style.baseWidgetSize * 0.8
        tooltipText: "Network Settings"
        onClicked: SettingsPanelService.openToTab(SettingsPanel.Tab.Connections, 0, screen)
      }

      // WiFi toggle (WiFi tab only)
      NToggle {
        id: wifiSwitch
        visible: root.currentTab === 0
        checked: Settings.data.network.wifiEnabled
        enabled: !Settings.data.network.airplaneModeEnabled && NetworkService.wifiAvailable
        onToggled: checked => NetworkService.setWifiEnabled(checked)
        baseSize: Style.baseWidgetSize * 0.65
      }
    }

    NDivider {
      Layout.fillWidth: true
    }

    // Error message
    Rectangle {
      visible: NetworkService.lastError.length > 0
      Layout.fillWidth: true
      Layout.preferredHeight: errorRow.implicitHeight + Style.marginM
      color: Qt.alpha(Color.mError, 0.1)
      radius: Style.radiusS
      border.width: Style.borderS
      border.color: Color.mError

      RowLayout {
        id: errorRow
        anchors.fill: parent
        anchors.margins: Style.marginS
        spacing: Style.marginS

        NIcon {
          icon: "warning"
          pointSize: Style.fontSizeM
          color: Color.mError
        }

        NText {
          text: NetworkService.lastError
          color: Color.mError
          pointSize: Style.fontSizeXS
          wrapMode: Text.Wrap
          Layout.fillWidth: true
        }

        NIconButton {
          icon: "close"
          baseSize: Style.baseWidgetSize * 0.5
          onClicked: NetworkService.lastError = ""
        }
      }
    }

    // WiFi disabled state (WiFi tab only)
    ColumnLayout {
      visible: root.currentTab === 0 && !Settings.data.network.wifiEnabled
      Layout.fillWidth: true
      spacing: Style.marginM

      NIcon {
        icon: "wifi-off"
        pointSize: Style.fontSizeXXL
        color: Color.mOnSurfaceVariant
        Layout.alignment: Qt.AlignHCenter
      }

      NText {
        text: "Wi-Fi is disabled"
        pointSize: Style.fontSizeM
        color: Color.mOnSurfaceVariant
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
      }

      NText {
        text: "Enable Wi-Fi to see available networks"
        pointSize: Style.fontSizeXS
        color: Color.mOnSurfaceVariant
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
      }
    }

    // WiFi Tab Content
    ColumnLayout {
      visible: root.currentTab === 0
      Layout.fillWidth: true
      spacing: Style.marginM

      // Scanning state
      ColumnLayout {
        visible: Settings.data.network.wifiEnabled && Object.keys(NetworkService.networks).length === 0 && NetworkService.scanning
        Layout.fillWidth: true
        spacing: Style.marginM

        NBusyIndicator {
          running: visible
          color: Color.mPrimary
          size: Style.baseWidgetSize * 0.8
          Layout.alignment: Qt.AlignHCenter
        }

        NText {
          text: "Searching for networks..."
          pointSize: Style.fontSizeS
          color: Color.mOnSurfaceVariant
          horizontalAlignment: Text.AlignHCenter
          Layout.fillWidth: true
        }
      }

      // Empty state
      ColumnLayout {
        visible: Settings.data.network.wifiEnabled && !NetworkService.scanning && Object.keys(NetworkService.networks).length === 0
        Layout.fillWidth: true
        spacing: Style.marginM

        NIcon {
          icon: "search"
          pointSize: Style.fontSizeXXL
          color: Color.mOnSurfaceVariant
          Layout.alignment: Qt.AlignHCenter
        }

        NText {
          text: "No networks found"
          pointSize: Style.fontSizeM
          color: Color.mOnSurfaceVariant
          horizontalAlignment: Text.AlignHCenter
          Layout.fillWidth: true
        }

        NButton {
          text: "Scan again"
          icon: "refresh"
          Layout.alignment: Qt.AlignHCenter
          onClicked: NetworkService.scan()
        }
      }

      // Networks list
      NScrollView {
        visible: Settings.data.network.wifiEnabled && Object.keys(NetworkService.networks).length > 0
        Layout.fillWidth: true
        Layout.preferredHeight: Math.min(networksColumn.implicitHeight, Math.round(250 * Style.uiScaleRatio))
        horizontalPolicy: ScrollBar.AlwaysOff
        verticalPolicy: ScrollBar.AsNeeded
        reserveScrollbarSpace: false

        ColumnLayout {
          id: networksColumn
          width: parent.width
          spacing: Style.marginS

          // Known networks section
          WiFiNetworksList {
            visible: root.knownNetworks.length > 0
            label: "Known networks"
            model: root.knownNetworks
            passwordSsid: root.passwordSsid
            expandedSsid: root.expandedSsid
            onPasswordRequested: ssid => {
              root.passwordSsid = ssid;
              root.expandedSsid = "";
            }
            onPasswordSubmitted: (ssid, password) => {
              NetworkService.connect(ssid, password);
              root.passwordSsid = "";
            }
            onPasswordCancelled: root.passwordSsid = ""
            onForgetRequested: ssid => root.expandedSsid = root.expandedSsid === ssid ? "" : ssid
            onForgetConfirmed: ssid => {
              NetworkService.forget(ssid);
              root.expandedSsid = "";
            }
            onForgetCancelled: root.expandedSsid = ""
          }

          // Available networks section
          WiFiNetworksList {
            visible: root.availableNetworks.length > 0
            label: "Available networks"
            model: root.availableNetworks
            passwordSsid: root.passwordSsid
            expandedSsid: root.expandedSsid
            onPasswordRequested: ssid => {
              root.passwordSsid = ssid;
              root.expandedSsid = "";
            }
            onPasswordSubmitted: (ssid, password) => {
              NetworkService.connect(ssid, password);
              root.passwordSsid = "";
            }
            onPasswordCancelled: root.passwordSsid = ""
            onForgetRequested: ssid => root.expandedSsid = root.expandedSsid === ssid ? "" : ssid
            onForgetConfirmed: ssid => {
              NetworkService.forget(ssid);
              root.expandedSsid = "";
            }
            onForgetCancelled: root.expandedSsid = ""
          }
        }
      }
    }

    // Ethernet Tab Content
    ColumnLayout {
      visible: root.currentTab === 1
      Layout.fillWidth: true
      spacing: Style.marginM

      // Ethernet status
      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginS

        NIcon {
          pointSize: Style.fontSizeXL
          color: NetworkService.ethernetConnected ? Color.mPrimary : Color.mOnSurfaceVariant
          icon: NetworkService.ethernetConnected ? (NetworkService.internetConnectivity ? "ethernet" : "ethernet-off") : "ethernet-off"
        }

        ColumnLayout {
          Layout.fillWidth: true
          spacing: 0

          NText {
            text: NetworkService.ethernetConnected ? "Ethernet Connected" : "Not Connected"
            font.weight: Style.fontWeightBold
            pointSize: Style.fontSizeS
          }

          NText {
            visible: NetworkService.ethernetConnected
            text: {
              const d = NetworkService.activeEthernetDetails || ({});
              const speed = (d.speed && d.speed.length > 0) ? d.speed : "";
              return speed ? "Speed: " + speed : "Connected";
            }
            pointSize: Style.fontSizeXS
            color: Color.mOnSurfaceVariant
          }
        }
      }

      // Not connected state
      ColumnLayout {
        visible: !NetworkService.ethernetConnected
        Layout.fillWidth: true
        spacing: Style.marginS

        NIcon {
          icon: "ethernet-off"
          pointSize: Style.fontSizeXXL
          color: Color.mOnSurfaceVariant
          Layout.alignment: Qt.AlignHCenter
        }

        NText {
          text: "No Ethernet connection"
          pointSize: Style.fontSizeS
          color: Color.mOnSurfaceVariant
          horizontalAlignment: Text.AlignHCenter
          Layout.fillWidth: true
        }

        NText {
          text: "Connect an Ethernet cable to get started"
          pointSize: Style.fontSizeXS
          color: Color.mOnSurfaceVariant
          horizontalAlignment: Text.AlignHCenter
          Layout.fillWidth: true
        }
      }
    }
  }
}
