import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.Media
import qs.Services.System
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property real localVolume: AudioService.volume

  Connections {
    target: AudioService
    function onSinkChanged() {
      localVolume = AudioService.volume;
    }
    function onVolumeChanged() {
      localVolume = AudioService.volume;
    }
  }

  Connections {
    target: AudioService.sink?.audio ? AudioService.sink?.audio : null
    function onVolumeChanged() {
      localVolume = AudioService.volume;
    }
  }

  // Output Volume
  ColumnLayout {
    spacing: Style.marginXXS
    Layout.fillWidth: true

    NValueSlider {
      Layout.fillWidth: true
      label: "Output volume"
      description: "System-wide volume level."
      from: 0
      to: Settings.data.audio.volumeOverdrive ? 1.5 : 1.0
      value: localVolume
      stepSize: 0.01
      text: Math.round(AudioService.volume * 100) + "%"
      onMoved: value => localVolume = value
    }

    Timer {
      interval: 100
      running: true
      repeat: true
      onTriggered: {
        if (!AudioService.isSwitchingSink && Math.abs(localVolume - AudioService.volume) >= 0.01) {
          AudioService.setVolume(localVolume);
        }
      }
    }
  }

  // Mute Toggle
  ColumnLayout {
    spacing: Style.marginS
    Layout.fillWidth: true

    NToggle {
      label: "Mute audio output"
      description: "Mute the system's main audio output."
      checked: AudioService.muted
      onToggled: checked => AudioService.setOutputMuted(checked)
    }
  }

  // Volume Feedback sound Toggle
  ColumnLayout {
    spacing: Style.marginS
    Layout.fillWidth: true

    NToggle {
      label: "Play volume feedback sound"
      description: "Play a feedback sound when adjusting volume."
      checked: Settings.data.audio.volumeFeedback
      defaultValue: Settings.getDefaultValue("audio.volumeFeedback")
      onToggled: checked => Settings.data.audio.volumeFeedback = checked
    }

    ColumnLayout {
      enabled: SoundService.multimediaAvailable && Settings.data.audio.volumeFeedback
      spacing: Style.marginXXS
      Layout.fillWidth: true

      NLabel {
        label: "Volume feedback sound file"
        description: "Path to the sound file played when changing volume."
      }

      NTextInputButton {
        enabled: parent.enabled
        Layout.fillWidth: true
        placeholderText: "Enter path to sound file"
        text: Settings.data.audio.volumeFeedbackSoundFile ?? ""
        buttonIcon: "folder-open"
        buttonTooltip: "Select sound file"
        onInputEditingFinished: Settings.data.audio.volumeFeedbackSoundFile = text
        onButtonClicked: volumeFeedbackFilePicker.open()
      }
    }
  }

  NDivider {
    Layout.fillWidth: true
  }

  // Input Volume
  ColumnLayout {
    spacing: Style.marginXS
    Layout.fillWidth: true

    NValueSlider {
      Layout.fillWidth: true
      label: "Input volume"
      description: "Microphone input volume level."
      from: 0
      to: Settings.data.audio.volumeOverdrive ? 1.5 : 1.0
      value: AudioService.inputVolume
      stepSize: 0.01
      text: Math.round(AudioService.inputVolume * 100) + "%"
      onMoved: value => AudioService.setInputVolume(value)
    }
  }

  // Input Mute Toggle
  ColumnLayout {
    spacing: Style.marginS
    Layout.fillWidth: true

    NToggle {
      label: "Mute audio input"
      description: "Mute the default audio input (microphone)."
      checked: AudioService.inputMuted
      onToggled: checked => AudioService.setInputMuted(checked)
    }
  }

  // Volume Step Size
  ColumnLayout {
    spacing: Style.marginS
    Layout.fillWidth: true

    NSpinBox {
      Layout.fillWidth: true
      label: "Volume step size"
      description: "Adjust the step size for volume changes (scroll wheel, keyboard shortcuts)."
      minimum: 1
      maximum: 25
      value: Settings.data.audio.volumeStep
      stepSize: 1
      suffix: "%"
      defaultValue: Settings.getDefaultValue("audio.volumeStep")
      onValueChanged: Settings.data.audio.volumeStep = value
    }
  }

  NDivider {
    Layout.fillWidth: true
  }

  // Raise maximum volume above 100%
  ColumnLayout {
    spacing: Style.marginS
    Layout.fillWidth: true

    NToggle {
      label: "Allow volume overdrive"
      description: "Allow raising volume above 100%. May not be supported by all hardware."
      checked: Settings.data.audio.volumeOverdrive
      defaultValue: Settings.getDefaultValue("audio.volumeOverdrive")
      onToggled: checked => Settings.data.audio.volumeOverdrive = checked
    }
  }

  NFilePicker {
    id: volumeFeedbackFilePicker
    title: "Select volume feedback sound file"
    selectionMode: "files"
    initialPath: Quickshell.env("HOME")
    nameFilters: ["*.wav", "*.mp3", "*.ogg", "*.flac", "*.m4a", "*.aac"]
    onAccepted: paths => {
                  if (paths.length > 0) {
                    Settings.data.audio.volumeFeedbackSoundFile = paths[0];
                  }
                }
  }
}
