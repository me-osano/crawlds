import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.System
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NToggle {
    label: "Enable clipboard history"
    description: "Access previously copied items from the launcher."
    checked: Settings.data.appLauncher.enableClipboardHistory
    onToggled: checked => Settings.data.appLauncher.enableClipboardHistory = checked
    defaultValue: Settings.getDefaultValue("appLauncher.enableClipboardHistory")
  }

  NToggle {
    label: "Enable clip preview"
    description: "Show a preview of the clipboard content when using the >clip command."
    checked: Settings.data.appLauncher.enableClipPreview
    onToggled: checked => Settings.data.appLauncher.enableClipPreview = checked
    defaultValue: Settings.getDefaultValue("appLauncher.enableClipPreview")
    enabled: Settings.data.appLauncher.enableClipboardHistory
  }

  NToggle {
    label: "Wrap clipboard text"
    description: "Wrap text in the clipboard list instead of truncating it."
    checked: Settings.data.appLauncher.clipboardWrapText
    onToggled: checked => Settings.data.appLauncher.clipboardWrapText = checked
    defaultValue: Settings.getDefaultValue("appLauncher.clipboardWrapText")
    enabled: Settings.data.appLauncher.enableClipboardHistory
  }

  NToggle {
    label: "Auto paste"
    description: "Automatically paste the selected clipboard item. Requires wtype."
    checked: Settings.data.appLauncher.autoPasteClipboard
    onToggled: checked => Settings.data.appLauncher.autoPasteClipboard = checked
    defaultValue: Settings.getDefaultValue("appLauncher.autoPasteClipboard")
    enabled: Settings.data.appLauncher.enableClipboardHistory && ProgramCheckerService.wtypeAvailable
  }

  NDivider {
    Layout.fillWidth: true
    visible: Settings.data.appLauncher.enableClipboardHistory
  }

  NTextInput {
    label: "Text watch command"
    description: "Full command string passed to wl-paste for text changes. (requires restart)"
    Layout.fillWidth: true
    text: Settings.data.appLauncher.clipboardWatchTextCommand
    onEditingFinished: Settings.data.appLauncher.clipboardWatchTextCommand = text
    enabled: Settings.data.appLauncher.enableClipboardHistory
    visible: Settings.data.appLauncher.enableClipboardHistory
  }

  NTextInput {
    label: "Image watch command"
    description: "Full command string passed to wl-paste for image changes. (requires restart)"
    Layout.fillWidth: true
    text: Settings.data.appLauncher.clipboardWatchImageCommand
    onEditingFinished: Settings.data.appLauncher.clipboardWatchImageCommand = text
    enabled: Settings.data.appLauncher.enableClipboardHistory
    visible: Settings.data.appLauncher.enableClipboardHistory
  }
}
