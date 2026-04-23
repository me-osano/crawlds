import QtQuick
import QtQuick.Controls
import QtQuick.Effects
import QtQuick.Layouts
import Quickshell
import Quickshell.Widgets
import qs.Common
import qs.Modules.Bar.Extras
import qs.Modules.Settings
import qs.Services.Core
import qs.Services.System
import qs.Services.UI
import qs.Widgets

Item {
  id: root

  property ShellScreen screen

  // Widget properties passed from Bar.qml for per-instance settings
  property string widgetId: ""
  property string section: ""
  property int sectionWidgetIndex: -1
  property int sectionWidgetsCount: 0

  property var widgetMetadata: BarWidgetRegistry.widgetMetadata[widgetId]
  // Explicit screenName property ensures reactive binding when screen changes
  readonly property string screenName: screen ? screen.name : ""
  property var widgetSettings: {
    if (section && sectionWidgetIndex >= 0 && screenName) {
      var widgets = Settings.getBarWidgetsForScreen(screenName)[section];
      if (widgets && sectionWidgetIndex < widgets.length) {
        return widgets[sectionWidgetIndex];
      }
    }
    return {};
  }

  readonly property string barPosition: Settings.getBarPositionForScreen(screenName)
  readonly property bool isBarVertical: barPosition === "left" || barPosition === "right"
  readonly property real capsuleHeight: Style.getCapsuleHeightForScreen(screenName)

  // Display mode: "icon" (single icon) or "capsule" (multi-icon capsule)
  readonly property string displayMode: widgetSettings.displayMode !== undefined ? widgetSettings.displayMode : widgetMetadata.displayMode
  readonly property bool isCapsuleMode: displayMode === "capsule"

  // Which icons to show in capsule mode
  readonly property bool showNetworkIcon: widgetSettings.showNetworkIcon !== undefined ? widgetSettings.showNetworkIcon : widgetMetadata.showNetworkIcon
  readonly property bool showBluetoothIcon: widgetSettings.showBluetoothIcon !== undefined ? widgetSettings.showBluetoothIcon : widgetMetadata.showBluetoothIcon
  readonly property bool showNotificationIcon: widgetSettings.showNotificationIcon !== undefined ? widgetSettings.showNotificationIcon : widgetMetadata.showNotificationIcon

  // Icon mode settings
  readonly property string customIcon: widgetSettings.icon !== undefined ? widgetSettings.icon : widgetMetadata.icon
  readonly property bool useDistroLogo: widgetSettings.useDistroLogo !== undefined ? widgetSettings.useDistroLogo : widgetMetadata.useDistroLogo
  readonly property string customIconPath: widgetSettings.customIconPath !== undefined ? widgetSettings.customIconPath : widgetMetadata.customIconPath
  readonly property bool enableColorization: widgetSettings.enableColorization !== undefined ? widgetSettings.enableColorization : widgetMetadata.enableColorization
  readonly property string colorizeSystemIcon: widgetSettings.colorizeSystemIcon !== undefined ? widgetSettings.colorizeSystemIcon : widgetMetadata.colorizeSystemIcon

  readonly property color iconColor: {
    if (!enableColorization)
      return Color.mOnSurface;
    return Color.resolveColorKey(colorizeSystemIcon);
  }

  // Notification unread count
  function computeUnreadCount() {
    var since = NotificationService.lastSeenTs;
    var count = 0;
    var model = NotificationService.historyList;
    for (var i = 0; i < model.count; i++) {
      var item = model.get(i);
      var ts = item.timestamp instanceof Date ? item.timestamp.getTime() : item.timestamp;
      if (ts > since)
        count++;
    }
    return count;
  }
  readonly property int unreadCount: computeUnreadCount()

  // Size based on display mode
  implicitWidth: isCapsuleMode ? capsuleLoader.implicitWidth : iconButtonLoader.implicitWidth
  implicitHeight: isCapsuleMode ? capsuleLoader.implicitHeight : iconButtonLoader.implicitHeight

  NPopupContextMenu {
    id: contextMenu

    model: [
      {
        "label": "Open launcher",
        "action": "open-launcher",
        "icon": "search"
      },
      {
        "label": "Open settings",
        "action": "open-settings",
        "icon": "adjustments"
      },
      {
        "label": "Widget settings",
        "action": "widget-settings",
        "icon": "settings"
      },
    ]

    onTriggered: action => {
                   contextMenu.close();
                   PanelService.closeContextMenu(screen);

                   if (action === "open-launcher") {
                     PanelService.toggleLauncher(screen);
                   } else if (action === "open-settings") {
                     var panel = PanelService.getPanel("settingsPanel", screen);
                     panel.requestedTab = SettingsPanel.Tab.General;
                     panel.toggle();
                   } else if (action === "widget-settings") {
                     BarService.openWidgetSettings(screen, section, sectionWidgetIndex, widgetId, widgetSettings);
                   }
                 }
  }

  function openControlCenter(anchor) {
    var controlCenterPanel = PanelService.getPanel("controlCenterPanel", screen);
    if (Settings.data.controlCenter.position === "close_to_bar_button") {
      controlCenterPanel?.toggle(anchor);
    } else {
      controlCenterPanel?.toggle();
    }
  }

  // Capsule mode: multiple icons in a row
  Loader {
    id: capsuleLoader
    active: isCapsuleMode
    anchors.fill: parent
    sourceComponent: capsuleComponent
  }

  Component {
    id: capsuleComponent

    Rectangle {
      id: capsuleBg

      property bool hovered: capsuleMouseArea.containsMouse

      implicitWidth: isBarVertical ? capsuleHeight : capsuleRow.implicitWidth + Style.marginM * 2
      implicitHeight: isBarVertical ? capsuleRow.implicitHeight + Style.marginM * 2 : capsuleHeight
      radius: Style.radiusM
      color: hovered ? Color.mHover : Style.capsuleColor
      border.color: Style.capsuleBorderColor
      border.width: Style.capsuleBorderWidth

      Behavior on color {
        enabled: !Color.isTransitioning
        ColorAnimation {
          duration: Style.animationFast
          easing.type: Easing.InOutQuad
        }
      }

      MouseArea {
        id: capsuleMouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        cursorShape: Qt.PointingHandCursor

        onEntered: TooltipService.show(capsuleBg, "Control center", BarService.getTooltipDirection(screenName))
        onExited: TooltipService.hide()

        onClicked: mouse => {
          TooltipService.hide();
          if (mouse.button === Qt.RightButton) {
            PanelService.showContextMenu(contextMenu, capsuleBg, screen);
          } else if (mouse.button === Qt.MiddleButton) {
            PanelService.toggleLauncher(screen);
          } else {
            openControlCenter(capsuleBg);
          }
        }
      }

      RowLayout {
        id: capsuleRow
        visible: !isBarVertical
        anchors.centerIn: parent
        spacing: Style.marginS

        // Notification icon with badge
        Item {
          visible: showNotificationIcon
          implicitWidth: notifIcon.implicitWidth
          implicitHeight: notifIcon.implicitHeight

          NIcon {
            id: notifIcon
            icon: NotificationService.doNotDisturb ? "bell-off" : "bell"
            pointSize: capsuleHeight * 0.5
            color: capsuleBg.hovered ? Color.mOnHover : Color.mOnSurface
          }

          // Unread badge
          Rectangle {
            visible: unreadCount > 0
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.rightMargin: -2
            anchors.topMargin: -2
            width: 7
            height: 7
            radius: Style.radiusXS
            color: capsuleBg.hovered ? Color.mOnHover : Color.mError
            border.color: Style.capsuleColor
            border.width: Style.borderS
          }
        }

        // Bluetooth icon
        NIcon {
          visible: showBluetoothIcon
          icon: !BluetoothService.enabled ? "bluetooth-off" : ((BluetoothService.connectedDevices && BluetoothService.connectedDevices.length > 0) ? "bluetooth-connected" : "bluetooth")
          pointSize: capsuleHeight * 0.5
          color: capsuleBg.hovered ? Color.mOnHover : Color.mOnSurface
        }

        // Network icon
        NIcon {
          visible: showNetworkIcon
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
              return "wifi-off";
            }
          }
          pointSize: capsuleHeight * 0.7
          color: capsuleBg.hovered ? Color.mOnHover : Color.mOnSurface
        }
      }

      // Vertical bar layout
      ColumnLayout {
        id: capsuleColumn
        visible: isBarVertical
        anchors.centerIn: parent
        spacing: Style.marginS

        // Network icon
        NIcon {
          visible: showNetworkIcon
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
              return "wifi-off";
            }
          }
          pointSize: capsuleHeight * 0.7
          color: capsuleBg.hovered ? Color.mOnHover : Color.mOnSurface
        }

        // Bluetooth icon
        NIcon {
          visible: showBluetoothIcon
          icon: !BluetoothService.enabled ? "bluetooth-off" : ((BluetoothService.connectedDevices && BluetoothService.connectedDevices.length > 0) ? "bluetooth-connected" : "bluetooth")
          pointSize: capsuleHeight * 0.5
          color: capsuleBg.hovered ? Color.mOnHover : Color.mOnSurface
        }

        // Notification icon with badge
        Item {
          visible: showNotificationIcon
          implicitWidth: notifIconV.implicitWidth
          implicitHeight: notifIconV.implicitHeight

          NIcon {
            id: notifIconV
            icon: NotificationService.doNotDisturb ? "bell-off" : "bell"
            pointSize: capsuleHeight * 0.5
            color: capsuleBg.hovered ? Color.mOnHover : Color.mOnSurface
          }

          // Unread badge
          Rectangle {
            visible: unreadCount > 0
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.rightMargin: -2
            anchors.topMargin: -2
            width: 7
            height: 7
            radius: Style.radiusXS
            color: capsuleBg.hovered ? Color.mOnHover : Color.mError
            border.color: Style.capsuleColor
            border.width: Style.borderS
          }
        }
      }
    }
  }

  // Icon mode: single button (original behavior)
  Loader {
    id: iconButtonLoader
    active: !isCapsuleMode
    anchors.fill: parent
    sourceComponent: iconButtonComponent
  }

  Component {
    id: iconButtonComponent

    NIconButton {
      id: iconButton

      icon: (customIconPath === "" && !useDistroLogo) ? customIcon : ""
      tooltipText: "Control center"
      tooltipDirection: BarService.getTooltipDirection(screenName)
      baseSize: capsuleHeight
      applyUiScale: false
      customRadius: Style.radiusL
      colorBg: Style.capsuleColor
      colorFg: iconColor
      colorBgHover: Color.mHover
      colorFgHover: Color.mOnHover
      colorBorder: Style.capsuleBorderColor
      colorBorderHover: Style.capsuleBorderColor

      onClicked: openControlCenter(iconButton)
      onRightClicked: PanelService.showContextMenu(contextMenu, iconButton, screen)
      onMiddleClicked: PanelService.toggleLauncher(screen)

      IconImage {
        id: customOrDistroLogo
        anchors.centerIn: parent
        width: iconButton.buttonSize * 0.8
        height: width
        source: {
          if (useDistroLogo)
            return HostService.osLogo;
          if (customIconPath !== "")
            return customIconPath.startsWith("file://") ? customIconPath : "file://" + customIconPath;
          return "";
        }
        visible: source !== ""
        smooth: true
        asynchronous: true
        layer.enabled: (enableColorization) && (useDistroLogo || customIconPath !== "")
        layer.effect: ShaderEffect {
          property color targetColor: !iconButton.hovering ? iconColor : Color.mOnHover
          property real colorizeMode: 2.0

          fragmentShader: Qt.resolvedUrl(Quickshell.shellDir + "/Common/Shaders/qsb/appicon_colorize.frag.qsb")
        }
      }
    }
  }
}
