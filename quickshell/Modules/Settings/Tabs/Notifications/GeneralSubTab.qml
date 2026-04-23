import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.Compositor
import qs.Services.System
import qs.Widgets

ColumnLayout {
  id: root

  Layout.fillWidth: true

  property var addMonitor
  property var removeMonitor

  NToggle {
    label: "Enable notifications"
    description: "Enable or disable the notification daemon, requires a restart of CrawlDS shell."
    checked: Settings.data.notifications.enabled !== false
    onToggled: checked => Settings.data.notifications.enabled = checked
    defaultValue: Settings.getDefaultValue("notifications.enabled")
  }

  ColumnLayout {
    spacing: Style.marginL
    enabled: Settings.data.notifications.enabled

    NComboBox {
      label: "Density"
      description: "Choose the notification card density."
      model: [
        {
          "key": "default",
          "name": "Default"
        },
        {
          "key": "compact",
          "name": "Compact"
        }
      ]
      currentKey: Settings.data.notifications.density || "default"
      onSelected: key => Settings.data.notifications.density = key
      defaultValue: Settings.getDefaultValue("notifications.density")
    }

    NToggle {
      label: "Do Not Disturb"
      description: "Disable all notification popups when enabled."
      checked: NotificationService.doNotDisturb
      onToggled: checked => NotificationService.doNotDisturb = checked
    }

    NComboBox {
      label: "Position"
      description: "Where notifications appear on screen."
      model: [
        {
          "key": "top",
          "name": "Top center"
        },
        {
          "key": "top_left",
          "name": "Top left"
        },
        {
          "key": "top_right",
          "name": "Top right"
        },
        {
          "key": "bottom",
          "name": "Bottom center"
        },
        {
          "key": "bottom_left",
          "name": "Bottom left"
        },
        {
          "key": "bottom_right",
          "name": "Bottom right"
        }
      ]
      currentKey: Settings.data.notifications.location || "top_right"
      onSelected: key => Settings.data.notifications.location = key
      defaultValue: Settings.getDefaultValue("notifications.location")
    }

    NToggle {
      label: "Always on top"
      description: "Display notifications above fullscreen windows and other layers."
      checked: Settings.data.notifications.overlayLayer
      onToggled: checked => Settings.data.notifications.overlayLayer = checked
      defaultValue: Settings.getDefaultValue("notifications.overlayLayer")
    }

    NValueSlider {
      Layout.fillWidth: true
      label: "Background opacity"
      description: "Adjust the opacity of notification backgrounds."
      from: 0
      to: 1
      stepSize: 0.01
      showReset: true
      value: Settings.data.notifications.backgroundOpacity
      onMoved: value => Settings.data.notifications.backgroundOpacity = value
      text: Math.round(Settings.data.notifications.backgroundOpacity * 100) + "%"
      defaultValue: Settings.getDefaultValue("notifications.backgroundOpacity")
    }

    NDivider {
      Layout.fillWidth: true
    }

    NText {
      text: "Show notification on specific monitors. Defaults to all if none are chosen."
      wrapMode: Text.WordWrap
      Layout.fillWidth: true
    }

    Repeater {
      model: Quickshell.screens || []
      delegate: NCheckbox {
        Layout.fillWidth: true
        label: modelData.name || "Unknown"
        description: {
          const compositorScale = CompositorService.getDisplayScale(modelData.name);
          "{model} ({width}x{height} @ {scale}x)";
        }
        checked: (Settings.data.notifications.monitors || []).indexOf(modelData.name) !== -1
        onToggled: checked => {
                     if (checked) {
                       Settings.data.notifications.monitors = root.addMonitor(Settings.data.notifications.monitors, modelData.name);
                     } else {
                       Settings.data.notifications.monitors = root.removeMonitor(Settings.data.notifications.monitors, modelData.name);
                     }
                   }
      }
    }
  }
}
