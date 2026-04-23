import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

// Session Menu Entry Settings Dialog Component
Popup {
  id: root

  property int entryIndex: -1
  property var entryData: null
  property string entryId: ""
  property string entryText: ""
  property string keybindInputText: ""

  signal updateEntryProperties(int index, var properties)

  // Default commands mapping
  readonly property var defaultCommands: {
    "lock": "Internal lock screen (no command)",
    "suspend": "systemctl suspend || loginctl suspend",
    "hibernate": "systemctl hibernate || loginctl hibernate",
    "reboot": "systemctl reboot || loginctl reboot",
    "rebootToUefi": "systemctl reboot --firmware-setup || loginctl reboot --firmware-setup",
    "logout": "Internal logout (no command)",
    "shutdown": "systemctl poweroff || loginctl poweroff"
  }

  readonly property string defaultCommand: defaultCommands[entryId] || ""

  width: Math.max(content.implicitWidth + padding * 2, 500)
  height: content.implicitHeight + padding * 2
  padding: Style.marginXL
  modal: true
  dim: false
  anchors.centerIn: parent

  onOpened: {
    // Load command when popup opens
    if (entryData) {
      commandInput.text = entryData.command || "";
      keybindInputText = entryData.keybind || "";
    }
    // Request focus to ensure keyboard input works
    forceActiveFocus();
  }

  function save() {
    root.updateEntryProperties(root.entryIndex, {
                                 "command": commandInput.text,
                                 "keybind": keybindInputText
                               });
  }

  background: Rectangle {
    id: bgRect

    color: Color.mSurface
    radius: Style.radiusL
    border.color: Color.mPrimary
    border.width: Style.borderM
  }

  contentItem: FocusScope {
    id: focusScope
    focus: true

    ColumnLayout {
      id: content
      anchors.fill: parent
      spacing: Style.marginM

      // Title
      RowLayout {
        Layout.fillWidth: true

        NText {
          text: "Configure {entry}"
          pointSize: Style.fontSizeL
          font.weight: Style.fontWeightBold
          color: Color.mPrimary
          Layout.fillWidth: true
        }

        NIconButton {
          icon: "close"
          tooltipText: "Close"
          onClicked: {
            root.save();
            root.close();
          }
        }
      }

      // Separator
      Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: 1
        color: Color.mOutline
      }

      // Command input
      NTextInput {
        id: commandInput
        Layout.fillWidth: true
        label: "Command"
        description: "Custom command to execute for this action. Leave empty to use the default system command."
        placeholderText: "e.g. systemctl poweroff"
        onTextChanged: root.save()
      }

      // Default command info
      ColumnLayout {
        Layout.fillWidth: true
        spacing: Style.marginXS

        NLabel {
          label: "Default command"
          description: "If no custom command is specified, the default system command will be used."
          Layout.fillWidth: true
        }

        // Default command display
        Rectangle {
          Layout.fillWidth: true
          Layout.preferredHeight: defaultCommandText.implicitHeight + Style.margin2M
          radius: Style.radiusM
          color: Color.mSurfaceVariant
          border.color: Color.mOutline
          border.width: Style.borderS

          RowLayout {
            anchors.fill: parent
            anchors.margins: Style.marginM
            spacing: Style.marginS

            NIcon {
              icon: "info"
              color: Color.mOnSurfaceVariant
              pointSize: Style.fontSizeM
            }

            NText {
              id: defaultCommandText
              Layout.fillWidth: true
              text: root.defaultCommand
              color: Color.mOnSurfaceVariant
              font.family: "monospace"
              font.pointSize: Style.fontSizeS
              wrapMode: Text.Wrap
            }
          }
        }
      }

      NKeybindRecorder {
        id: keybindRecorder
        Layout.fillWidth: true
        label: "Keybind"
        description: "Recording keybind..."
        allowEmpty: true
        maxKeybinds: 1
        requireModifierForNormalKeys: false
        currentKeybinds: keybindInputText ? [keybindInputText] : []
        settingsPath: "sessionMenu.powerOptions[" + root.entryIndex + "].keybind"
        onKeybindsChanged: newKeybinds => {
                             keybindInputText = newKeybinds.length > 0 ? newKeybinds[0] : "";
                             root.save();
                           }
      }

      // Hidden property to store the text since NKeybindRecorder manages its own state
      // but we need to initialize it and read from it

      // Bottom spacer to maintain padding
      Item {
        Layout.preferredHeight: Style.marginS
      }
    }
  }
}
