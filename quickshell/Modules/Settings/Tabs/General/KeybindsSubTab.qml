import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  width: parent.width

  // Keybinds section
  NLabel {
    label: "Navigation keybinds"
    description: "Configure global navigation keys for panels and launcher."
    Layout.fillWidth: true
  }

  NKeybindRecorder {
    Layout.fillWidth: true
    label: "Move up"
    currentKeybinds: Settings.data.general.keybinds.keyUp
    defaultKeybind: "Up"
    settingsPath: "general.keybinds.keyUp"
    onKeybindsChanged: newKeybinds => Settings.data.general.keybinds.keyUp = newKeybinds
  }

  NKeybindRecorder {
    Layout.fillWidth: true
    label: "Move down"
    currentKeybinds: Settings.data.general.keybinds.keyDown
    defaultKeybind: "Down"
    settingsPath: "general.keybinds.keyDown"
    onKeybindsChanged: newKeybinds => Settings.data.general.keybinds.keyDown = newKeybinds
  }

  NKeybindRecorder {
    Layout.fillWidth: true
    label: "Move left"
    currentKeybinds: Settings.data.general.keybinds.keyLeft
    defaultKeybind: "Left"
    settingsPath: "general.keybinds.keyLeft"
    onKeybindsChanged: newKeybinds => Settings.data.general.keybinds.keyLeft = newKeybinds
  }

  NKeybindRecorder {
    Layout.fillWidth: true
    label: "Move right"
    currentKeybinds: Settings.data.general.keybinds.keyRight
    defaultKeybind: "Right"
    settingsPath: "general.keybinds.keyRight"
    onKeybindsChanged: newKeybinds => Settings.data.general.keybinds.keyRight = newKeybinds
  }

  NKeybindRecorder {
    Layout.fillWidth: true
    label: "Confirm / Action"
    currentKeybinds: Settings.data.general.keybinds.keyEnter
    defaultKeybind: "Return"
    settingsPath: "general.keybinds.keyEnter"
    onKeybindsChanged: newKeybinds => Settings.data.general.keybinds.keyEnter = newKeybinds
  }

  NKeybindRecorder {
    Layout.fillWidth: true
    label: "Close / Back"
    currentKeybinds: Settings.data.general.keybinds.keyEscape
    defaultKeybind: "Esc"
    settingsPath: "general.keybinds.keyEscape"
    onKeybindsChanged: newKeybinds => Settings.data.general.keybinds.keyEscape = newKeybinds
  }

  NKeybindRecorder {
    Layout.fillWidth: true
    label: "Remove / Delete"
    currentKeybinds: Settings.data.general.keybinds.keyRemove
    defaultKeybind: "Del"
    settingsPath: "general.keybinds.keyRemove"
    onKeybindsChanged: newKeybinds => Settings.data.general.keybinds.keyRemove = newKeybinds
  }
}
