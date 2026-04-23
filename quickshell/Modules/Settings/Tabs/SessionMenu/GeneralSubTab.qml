import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NToggle {
    Layout.fillWidth: true
    label: "Large buttons style"
    description: "Display the session menu with large buttons in a grid layout."
    checked: Settings.data.sessionMenu.largeButtonsStyle
    onToggled: checked => Settings.data.sessionMenu.largeButtonsStyle = checked
  }

  NComboBox {
    visible: Settings.data.sessionMenu.largeButtonsStyle
    Layout.fillWidth: true
    label: "Large buttons layout"
    description: "Choose how session menu buttons are displayed."
    model: [
      {
        "key": "grid",
        "name": "Grid"
      },
      {
        "key": "single-row",
        "name": "Single row"
      }
    ]
    currentKey: Settings.data.sessionMenu.largeButtonsLayout
    defaultValue: Settings.getDefaultValue("sessionMenu.largeButtonsLayout")
    onSelected: key => Settings.data.sessionMenu.largeButtonsLayout = key
  }

  NComboBox {
    label: "Position"
    description: "Choose where the session menu panel appears when opened."
    Layout.fillWidth: true
    model: [
      {
        "key": "center",
        "name": "Center"
      },
      {
        "key": "top_center",
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
        "key": "bottom_center",
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
    currentKey: Settings.data.sessionMenu.position
    onSelected: key => Settings.data.sessionMenu.position = key
    visible: !Settings.data.sessionMenu.largeButtonsStyle
    defaultValue: Settings.getDefaultValue("sessionMenu.position")
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show header"
    description: "Display the title and close button at the top of the session menu."
    checked: Settings.data.sessionMenu.showHeader
    onToggled: checked => Settings.data.sessionMenu.showHeader = checked
    visible: !Settings.data.sessionMenu.largeButtonsStyle
    defaultValue: Settings.getDefaultValue("sessionMenu.showHeader")
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show keybinds"
    description: "Display keybind hints on session options."
    checked: Settings.data.sessionMenu.showKeybinds
    onToggled: checked => Settings.data.sessionMenu.showKeybinds = checked
    defaultValue: Settings.getDefaultValue("sessionMenu.showKeybinds")
  }

  NToggle {
    Layout.fillWidth: true
    label: "Enable countdown timer"
    description: "Show a countdown timer before executing power actions."
    checked: Settings.data.sessionMenu.enableCountdown
    onToggled: checked => Settings.data.sessionMenu.enableCountdown = checked
    defaultValue: Settings.getDefaultValue("sessionMenu.enableCountdown")
  }

  NValueSlider {
    visible: Settings.data.sessionMenu.enableCountdown
    Layout.fillWidth: true
    label: "Countdown duration"
    description: "Set how long the countdown timer lasts before executing power actions."
    from: 1000
    to: 30000
    stepSize: 1000
    showReset: true
    value: Settings.data.sessionMenu.countdownDuration
    onMoved: value => Settings.data.sessionMenu.countdownDuration = value
    text: Math.round(Settings.data.sessionMenu.countdownDuration / 1000) + "s"
    defaultValue: Settings.getDefaultValue("sessionMenu.countdownDuration")
  }
}
