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

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.width = parseInt(widthInput.text) || widgetMetadata.width;
    settingsChanged(settings);
  }

  NTextInput {
    id: widthInput
    Layout.fillWidth: true
    label: "Width"
    description: "Spacing width in pixels."
    text: widgetData.width || widgetMetadata.width
    placeholderText: "Enter width in pixels"
    onEditingFinished: saveSettings()
  }
}
