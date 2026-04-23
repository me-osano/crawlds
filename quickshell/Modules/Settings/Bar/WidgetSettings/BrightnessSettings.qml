import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
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
  property string valueDisplayMode: widgetData.displayMode !== undefined ? widgetData.displayMode : widgetMetadata.displayMode
  property string valueIconColor: widgetData.iconColor !== undefined ? widgetData.iconColor : widgetMetadata.iconColor
  property string valueTextColor: widgetData.textColor !== undefined ? widgetData.textColor : widgetMetadata.textColor
  property bool valueApplyToAllMonitors: widgetData.applyToAllMonitors !== undefined ? widgetData.applyToAllMonitors : widgetMetadata.applyToAllMonitors

  readonly property bool hasMultipleMonitors: (Quickshell.screens || []).length > 1

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.displayMode = valueDisplayMode;
    settings.iconColor = valueIconColor;
    settings.textColor = valueTextColor;
    settings.applyToAllMonitors = valueApplyToAllMonitors;
    settingsChanged(settings);
  }

  NComboBox {
    label: "Display mode"
    description: "Choose how you'd like this value to appear."
    minimumWidth: 200
    model: [
      {
        "key": "onhover",
        "name": "On hover"
      },
      {
        "key": "alwaysShow",
        "name": "Always show"
      },
      {
        "key": "alwaysHide",
        "name": "Always hide"
      }
    ]
    currentKey: valueDisplayMode
    onSelected: key => {
                  valueDisplayMode = key;
                  saveSettings();
                }
  }

  NColorChoice {
    label: "Select icon color"
    currentKey: valueIconColor
    onSelected: key => {
                  valueIconColor = key;
                  saveSettings();
                }
  }

  NColorChoice {
    currentKey: valueTextColor
    onSelected: key => {
                  valueTextColor = key;
                  saveSettings();
                }
  }

  NToggle {
    visible: hasMultipleMonitors
    Layout.fillWidth: true
    label: "Apply scroll changes to all monitors"
    description: "Change brightness for every monitor when using the scroll wheel."
    checked: valueApplyToAllMonitors
    onToggled: checked => {
                 valueApplyToAllMonitors = checked;
                 saveSettings();
               }
    defaultValue: widgetMetadata.applyToAllMonitors
  }
}
