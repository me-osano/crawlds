import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell.Services.Pipewire
import qs.Common
import qs.Services.Media
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  // Output Devices
  ButtonGroup {
    id: sinks
  }

  ColumnLayout {
    spacing: Style.marginXS
    Layout.fillWidth: true
    Layout.bottomMargin: Style.marginL

    NLabel {
      label: "Output device"
      description: "Select the desired audio output device."
    }

    Repeater {
      model: AudioService.sinks
      NRadioButton {
        ButtonGroup.group: sinks
        required property PwNode modelData
        text: modelData.description
        checked: AudioService.sink?.id === modelData.id
        onClicked: {
          AudioService.setAudioSink(modelData);
        }
        Layout.fillWidth: true
      }
    }
  }

  // Input Devices
  ButtonGroup {
    id: sources
  }

  ColumnLayout {
    spacing: Style.marginXS
    Layout.fillWidth: true

    NLabel {
      label: "Input device"
      description: "Select the desired audio input device."
    }

    Repeater {
      model: AudioService.sources
      NRadioButton {
        ButtonGroup.group: sources
        required property PwNode modelData
        text: modelData.description
        checked: AudioService.source?.id === modelData.id
        onClicked: AudioService.setAudioSource(modelData)
        Layout.fillWidth: true
      }
    }
  }
}
