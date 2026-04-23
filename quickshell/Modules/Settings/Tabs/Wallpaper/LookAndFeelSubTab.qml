import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  enabled: Settings.data.wallpaper.enabled

  property var screen

  NComboBox {
    label: "Fill mode"
    description: "Select how the image should scale to match your monitor's resolution."
    model: WallpaperService.fillModeModel
    currentKey: Settings.data.wallpaper.fillMode
    onSelected: key => Settings.data.wallpaper.fillMode = key
    defaultValue: Settings.getDefaultValue("wallpaper.fillMode")
  }

  RowLayout {
    NLabel {
      label: "Fill color"
      description: "Choose a fill color that may appear behind the wallpaper."
      Layout.alignment: Qt.AlignTop
    }

    NColorPicker {
      screen: root.screen
      selectedColor: Settings.data.wallpaper.fillColor
      onColorSelected: color => Settings.data.wallpaper.fillColor = color
    }
  }

  NComboBox {
    label: "Transition type"
    description: "Animation type when switching between wallpapers."
    model: WallpaperService.transitionsModel
    currentKey: Settings.data.wallpaper.transitionType
    onSelected: key => Settings.data.wallpaper.transitionType = key
    defaultValue: Settings.getDefaultValue("wallpaper.transitionType")
  }

  NToggle {
    label: "Skip startup transition"
    description: "Skip the wallpaper animation when the shell starts."
    checked: Settings.data.wallpaper.skipStartupTransition
    onToggled: Settings.data.wallpaper.skipStartupTransition = checked
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Transition duration"
    description: "Duration of transition animations in seconds."
    from: 500
    to: 10000
    stepSize: 100
    value: Settings.data.wallpaper.transitionDuration
    onMoved: value => Settings.data.wallpaper.transitionDuration = value
    text: (Settings.data.wallpaper.transitionDuration / 1000).toFixed(1) + "s"
    defaultValue: Settings.getDefaultValue("wallpaper.transitionDuration")
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Soften transition edge"
    description: "Applies a soft, feathered effect to the edge of transitions."
    from: 0.0
    to: 1.0
    value: Settings.data.wallpaper.transitionEdgeSmoothness
    onMoved: value => Settings.data.wallpaper.transitionEdgeSmoothness = value
    text: Math.round(Settings.data.wallpaper.transitionEdgeSmoothness * 100) + "%"
    defaultValue: Settings.getDefaultValue("wallpaper.transitionEdgeSmoothness")
  }
}
