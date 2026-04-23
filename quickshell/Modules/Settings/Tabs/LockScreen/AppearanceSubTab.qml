import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  function insertToken(token) {
    if (formatInput.inputItem) {
      var input = formatInput.inputItem;
      var cursorPos = input.cursorPosition;
      var currentText = input.text;
      var newText = currentText.substring(0, cursorPos) + token + currentText.substring(cursorPos);
      input.text = newText + " ";
      input.cursorPosition = cursorPos + token.length + 1;
      input.forceActiveFocus();
    }
  }

  NComboBox {
    label: "Clock style"
    description: "Choose the visual style of the clock on the lock screen."
    model: [
      {
        "key": "analog",
        "name": "Analog"
      },
      {
        "key": "digital",
        "name": "Digital"
      },
      {
        "key": "custom",
        "name": "Custom"
      }
    ]
    currentKey: Settings.data.general.clockStyle
    onSelected: key => Settings.data.general.clockStyle = key
    defaultValue: Settings.getDefaultValue("general.clockStyle")
    z: 10
  }

  NTextInput {
    id: formatInput
    label: "Clock format"
    description: "Customize the clock format using date/time syntax tokens."
    text: Settings.data.general.clockFormat
    onTextChanged: Settings.data.general.clockFormat = text
    visible: Settings.data.general.clockStyle === "custom"
    defaultValue: Settings.getDefaultValue("general.clockFormat")
  }

  NDateTimeTokens {
    Layout.fillWidth: true
    Layout.preferredHeight: 300
    visible: Settings.data.general.clockStyle === "custom"
    onTokenClicked: token => root.insertToken(token)
  }

  NToggle {
    label: "Random password icons"
    description: "Cute icons used to hide your password."
    checked: Settings.data.general.passwordChars
    onToggled: checked => Settings.data.general.passwordChars = checked
    defaultValue: Settings.getDefaultValue("general.passwordChars")
  }

  NToggle {
    label: "Compact lock screen"
    description: "Show only the login input and system controls, hiding weather and media widgets."
    checked: Settings.data.general.compactLockScreen
    onToggled: checked => Settings.data.general.compactLockScreen = checked
    defaultValue: Settings.getDefaultValue("general.compactLockScreen")
  }

  NToggle {
    label: "Lockscreen animations"
    description: "Enable or disable lockscreen animations."
    checked: Settings.data.general.lockScreenAnimations
    onToggled: checked => Settings.data.general.lockScreenAnimations = checked
    defaultValue: Settings.getDefaultValue("general.lockScreenAnimations")
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Lock screen blur strength"
    description: "Applies a blur effect to the lock screen wallpaper."
    from: 0.0
    to: 1.0
    stepSize: 0.01
    showReset: true
    value: Settings.data.general.lockScreenBlur
    onMoved: value => Settings.data.general.lockScreenBlur = value
    text: ((Settings.data.general.lockScreenBlur) * 100).toFixed(0) + "%"
    defaultValue: Settings.getDefaultValue("general.lockScreenBlur")
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Lock screen tint strength"
    description: "Applies a tint overlay to the lock screen wallpaper."
    from: 0.0
    to: 1.0
    stepSize: 0.01
    showReset: true
    value: Settings.data.general.lockScreenTint
    onMoved: value => Settings.data.general.lockScreenTint = value
    text: ((Settings.data.general.lockScreenTint) * 100).toFixed(0) + "%"
    defaultValue: Settings.getDefaultValue("general.lockScreenTint")
  }
}
