import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.Compositor
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property var addMonitor
  property var removeMonitor

  NComboBox {
    label: "Position"
    description: "Where on-screen displays appear."
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
      },
      {
        "key": "left",
        "name": "Left center"
      },
      {
        "key": "right",
        "name": "Right center"
      }
    ]
    currentKey: Settings.data.osd.location || "top_right"
    defaultValue: Settings.getDefaultValue("osd.location")
    onSelected: key => Settings.data.osd.location = key
  }

  NToggle {
    label: "Enable on-screen display"
    description: "Show volume and brightness changes in real-time."
    checked: Settings.data.osd.enabled
    defaultValue: Settings.getDefaultValue("osd.enabled")
    onToggled: checked => Settings.data.osd.enabled = checked
  }

  NToggle {
    label: "Always on top"
    description: "Display OSD above fullscreen windows and other layers."
    checked: Settings.data.osd.overlayLayer
    defaultValue: Settings.getDefaultValue("osd.overlayLayer")
    onToggled: checked => Settings.data.osd.overlayLayer = checked
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Background opacity"
    description: "Controls the transparency of the OSD background."
    from: 0
    to: 100
    stepSize: 1
    showReset: true
    value: Settings.data.osd.backgroundOpacity * 100
    defaultValue: (Settings.getDefaultValue("osd.backgroundOpacity") || 1) * 100
    onMoved: value => Settings.data.osd.backgroundOpacity = value / 100
    text: Math.round(Settings.data.osd.backgroundOpacity * 100) + "%"
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Auto-hide after"
    description: "Adjust the time before OSD disappears."
    from: 500
    to: 5000
    stepSize: 100
    showReset: true
    value: Settings.data.osd.autoHideMs
    defaultValue: Settings.getDefaultValue("osd.autoHideMs")
    onMoved: value => Settings.data.osd.autoHideMs = value
    text: Math.round(Settings.data.osd.autoHideMs / 1000 * 10) / 10 + "s"
  }

  NDivider {
    Layout.fillWidth: true
  }

  NText {
    text: "Show OSD on specific monitors. Defaults to all if none are chosen."
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
      checked: (Settings.data.osd.monitors || []).indexOf(modelData.name) !== -1
      onToggled: checked => {
                   if (checked) {
                     Settings.data.osd.monitors = root.addMonitor(Settings.data.osd.monitors, modelData.name);
                   } else {
                     Settings.data.osd.monitors = root.removeMonitor(Settings.data.osd.monitors, modelData.name);
                   }
                 }
    }
  }
}
