import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NToggle {
    label: "Snap panels to edges"
    description: "Panels lock to the bar and screen edges, creating a seamless look with stylish inverted corners."
    checked: Settings.data.ui.panelsAttachedToBar
    defaultValue: Settings.getDefaultValue("ui.panelsAttachedToBar")
    onToggled: checked => Settings.data.ui.panelsAttachedToBar = checked
  }

  NToggle {
    visible: (Quickshell.screens.length > 1)
    label: "Allow panels on screens without a bar"
    description: "When enabled, panels can open on any screen. When disabled, panels will only open on screens that have a bar, which can reduce memory usage."
    checked: Settings.data.general.allowPanelsOnScreenWithoutBar
    defaultValue: Settings.getDefaultValue("general.allowPanelsOnScreenWithoutBar")
    onToggled: checked => Settings.data.general.allowPanelsOnScreenWithoutBar = checked
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Panel background opacity"
    description: "Set the background opacity for all panels (bar, launcher, settings, etc...)."
    from: 0
    to: 1
    stepSize: 0.01
    showReset: true
    value: Settings.data.ui.panelBackgroundOpacity
    defaultValue: Settings.getDefaultValue("ui.panelBackgroundOpacity")
    onMoved: value => Settings.data.ui.panelBackgroundOpacity = value
    text: Math.floor(Settings.data.ui.panelBackgroundOpacity * 100) + "%"
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Dimmed desktop opacity"
    description: "Set the opacity level for desktop dimming."
    from: 0
    to: 1
    stepSize: 0.01
    showReset: true
    value: Settings.data.general.dimmerOpacity
    defaultValue: Settings.getDefaultValue("general.dimmerOpacity")
    onMoved: value => Settings.data.general.dimmerOpacity = value
    text: Math.floor(Settings.data.general.dimmerOpacity * 100) + "%"
  }

  NDivider {
    Layout.fillWidth: true
  }

  NHeader {
    label: "Settings panel"
  }

  NComboBox {
    label: "Settings panel mode"
    description: "Choose settings layout (may require reopening)."
    Layout.fillWidth: true
    model: [
      {
        "key": "attached",
        "name": "Panel attached to bar"
      },
      {
        "key": "centered",
        "name": "Centered panel"
      },
      {
        "key": "window",
        "name": "Separate window"
      }
    ]
    currentKey: Settings.data.ui.settingsPanelMode
    defaultValue: Settings.getDefaultValue("ui.settingsPanelMode")
    onSelected: key => Settings.data.ui.settingsPanelMode = key
    minimumWidth: 220 * Style.uiScaleRatio
  }

  NToggle {
    label: "Sidebar card style"
    description: "Wraps the settings sidebar in a filled background with rounded corners."
    checked: Settings.data.ui.settingsPanelSideBarCardStyle
    defaultValue: Settings.getDefaultValue("ui.settingsPanelSideBarCardStyle")
    onToggled: checked => Settings.data.ui.settingsPanelSideBarCardStyle = checked
  }
}
