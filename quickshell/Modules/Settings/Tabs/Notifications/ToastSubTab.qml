import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NCheckbox {
    Layout.fillWidth: true
    label: "Media"
    description: "Show a toast when media playback state changes."
    checked: Settings.data.notifications.enableMediaToast
    onToggled: checked => Settings.data.notifications.enableMediaToast = checked
  }

  NCheckbox {
    Layout.fillWidth: true
    label: "Keyboard layout"
    description: "Show a toast when the keyboard layout changes."
    checked: Settings.data.notifications.enableKeyboardLayoutToast
    onToggled: checked => Settings.data.notifications.enableKeyboardLayoutToast = checked
  }

  NCheckbox {
    Layout.fillWidth: true
    label: "Battery warning"
    description: "Show a warning when the battery level falls below this percentage."
    checked: Settings.data.notifications.enableBatteryToast
    onToggled: checked => Settings.data.notifications.enableBatteryToast = checked
  }
}
