import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM

  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  property bool valueShowBackground: widgetData.showBackground !== undefined ? widgetData.showBackground : widgetMetadata.showBackground
  property string valueVisualizerType: (widgetData.visualizerType && widgetData.visualizerType !== "") ? widgetData.visualizerType : (widgetMetadata.visualizerType || "linear")
  property string valueHideMode: widgetData.hideMode !== undefined ? widgetData.hideMode : widgetMetadata.hideMode
  property bool valueShowButtons: widgetData.showButtons !== undefined ? widgetData.showButtons : (widgetMetadata.showButtons !== undefined ? widgetMetadata.showButtons : true)
  property bool valueShowAlbumArt: widgetData.showAlbumArt !== undefined ? widgetData.showAlbumArt : (widgetMetadata.showAlbumArt !== undefined ? widgetMetadata.showAlbumArt : true)
  property bool valueShowVisualizer: widgetData.showVisualizer !== undefined ? widgetData.showVisualizer : (widgetMetadata.showVisualizer !== undefined ? widgetMetadata.showVisualizer : true)
  property bool valueRoundedCorners: widgetData.roundedCorners !== undefined ? widgetData.roundedCorners : (widgetMetadata.roundedCorners !== undefined ? widgetMetadata.roundedCorners : true)

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.showBackground = valueShowBackground;
    settings.visualizerType = valueVisualizerType;
    settings.hideMode = valueHideMode;
    settings.showButtons = valueShowButtons;
    settings.showAlbumArt = valueShowAlbumArt;
    settings.showVisualizer = valueShowVisualizer;
    settings.roundedCorners = valueRoundedCorners;
    settingsChanged(settings);
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show background"
    description: "Show the background container."
    checked: valueShowBackground
    onToggled: checked => {
                 valueShowBackground = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Rounded corners"
    description: "Enable rounded corners on the widget edges."
    checked: valueRoundedCorners
    onToggled: checked => {
                 valueRoundedCorners = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show album art & title"
    description: "Show the album artwork and track information (title and artist)."
    checked: valueShowAlbumArt
    onToggled: checked => {
                 valueShowAlbumArt = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show visualizer"
    description: "Show the audio visualizer overlay."
    checked: valueShowVisualizer
    onToggled: checked => {
                 valueShowVisualizer = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show buttons"
    description: "Show media control buttons (play/pause, previous, next)."
    checked: valueShowButtons
    onToggled: checked => {
                 valueShowButtons = checked;
                 saveSettings();
               }
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Visualization type"
    description: "Choose a visualization type."
    enabled: valueShowVisualizer
    model: [
      {
        "key": "linear",
        "name": "Linear"
      },
      {
        "key": "mirrored",
        "name": "Mirrored"
      },
      {
        "key": "wave",
        "name": "Wave"
      }
    ]
    currentKey: valueVisualizerType
    onSelected: key => {
                  valueVisualizerType = key;
                  saveSettings();
                }
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Hiding mode"
    description: "Controls how the widget behaves when no media is playing."
    model: [
      {
        "key": "hidden",
        "name": "Hide when empty"
      },
      {
        "key": "idle",
        "name": "Hide when idle"
      },
      {
        "key": "visible",
        "name": "Always visible"
      }
    ]
    currentKey: valueHideMode
    onSelected: key => {
                  valueHideMode = key;
                  saveSettings();
                }
  }
}
