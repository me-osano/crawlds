import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM

  // Properties to receive data from parent
  property var screen: null
  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  // Local state
  property bool valueHideWhenIdle: widgetData.hideWhenIdle !== undefined ? widgetData.hideWhenIdle : widgetMetadata.hideWhenIdle
  property string valueColorName: widgetData.colorName !== undefined ? widgetData.colorName : widgetMetadata.colorName

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.width = parseInt(widthInput.text) || widgetMetadata.width;
    settings.hideWhenIdle = valueHideWhenIdle;
    settings.colorName = valueColorName;
    settingsChanged(settings);
  }

  NTextInput {
    id: widthInput
    Layout.fillWidth: true
    label: "Width"
    description: "Custom component width."
    text: widgetData.width || widgetMetadata.width
    placeholderText: "Enter width in pixels"
    onEditingFinished: saveSettings()
  }

  NColorChoice {
    Layout.fillWidth: true
    label: "Fill color"
    description: "Select the color for the visualizer."
    currentKey: root.valueColorName
    onSelected: key => {
                  root.valueColorName = key;
                  saveSettings();
                }
  }

  NToggle {
    label: "Hide when no media is playing"
    description: "When enabled, the visualizer is hidden unless a player is actively playing."
    checked: valueHideWhenIdle
    onToggled: checked => {
                 valueHideWhenIdle = checked;
                 saveSettings();
               }
  }
}
