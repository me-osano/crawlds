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

  readonly property bool isVerticalBar: Settings.data.bar.position === "left" || Settings.data.bar.position === "right"

  // Local state
  property string valueHideMode: "hidden"
  property bool valueOnlyActiveWorkspaces: widgetData.onlyActiveWorkspaces !== undefined ? widgetData.onlyActiveWorkspaces : widgetMetadata.onlyActiveWorkspaces
  property bool valueOnlySameOutput: widgetData.onlySameOutput !== undefined ? widgetData.onlySameOutput : widgetMetadata.onlySameOutput
  property bool valueColorizeIcons: widgetData.colorizeIcons !== undefined ? widgetData.colorizeIcons : widgetMetadata.colorizeIcons
  property bool valueShowTitle: isVerticalBar ? false : widgetData.showTitle !== undefined ? widgetData.showTitle : widgetMetadata.showTitle
  property bool valueSmartWidth: widgetData.smartWidth !== undefined ? widgetData.smartWidth : widgetMetadata.smartWidth
  property int valueMaxTaskbarWidth: widgetData.maxTaskbarWidth !== undefined ? widgetData.maxTaskbarWidth : widgetMetadata.maxTaskbarWidth
  property int valueTitleWidth: widgetData.titleWidth !== undefined ? widgetData.titleWidth : widgetMetadata.titleWidth
  property bool valueShowPinnedApps: widgetData.showPinnedApps !== undefined ? widgetData.showPinnedApps : widgetMetadata.showPinnedApps
  property real valueIconScale: widgetData.iconScale !== undefined ? widgetData.iconScale : widgetMetadata.iconScale

  Component.onCompleted: {
    if (widgetData && widgetData.hideMode !== undefined) {
      valueHideMode = widgetData.hideMode;
    } else if (widgetMetadata && widgetMetadata.hideMode !== undefined) {
      valueHideMode = widgetMetadata.hideMode;
    }
  }

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.hideMode = valueHideMode;
    settings.onlySameOutput = valueOnlySameOutput;
    settings.onlyActiveWorkspaces = valueOnlyActiveWorkspaces;
    settings.colorizeIcons = valueColorizeIcons;
    settings.showTitle = valueShowTitle;
    settings.smartWidth = valueSmartWidth;
    settings.maxTaskbarWidth = valueMaxTaskbarWidth;
    settings.titleWidth = parseInt(titleWidthInput.text) || widgetMetadata.titleWidth;
    settings.showPinnedApps = valueShowPinnedApps;
    settings.iconScale = valueIconScale;
    settingsChanged(settings);
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Hiding mode"
    description: "Controls how the widget behaves when there are no matching windows."
    model: [
      {
        "key": "visible",
        "name": "Always visible"
      },
      {
        "key": "hidden",
        "name": "Hide when empty"
      },
      {
        "key": "transparent",
        "name": "Transparent when empty"
      }
    ]
    currentKey: root.valueHideMode
    onSelected: key => {
                  root.valueHideMode = key;
                  saveSettings();
                }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Only from same monitor"
    description: "Show only apps from the monitor where the bar is located."
    checked: root.valueOnlySameOutput
    onToggled: checked => {
                 root.valueOnlySameOutput = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Only from active workspaces"
    description: "Show only apps from active workspaces."
    checked: root.valueOnlyActiveWorkspaces
    onToggled: checked => {
                 root.valueOnlyActiveWorkspaces = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Colorize icons"
    description: "Apply theme colors to taskbar icons."
    checked: root.valueColorizeIcons
    onToggled: checked => {
                 root.valueColorizeIcons = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show pinned apps"
    description: "Show pinned apps from the dock in the taskbar."
    checked: root.valueShowPinnedApps
    onToggled: checked => {
                 root.valueShowPinnedApps = checked;
                 saveSettings();
               }
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Icon scaling"
    description: "Sets the scaling factor for taskbar icons."
    from: 0.5
    to: 1
    stepSize: 0.01
    value: root.valueIconScale
    onMoved: value => {
               root.valueIconScale = value;
               saveSettings();
             }
    text: Math.round(root.valueIconScale * 100) + "%"
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show title"
    description: isVerticalBar ? "Vertical taskbar does not support showing titles." : "Display window titles in the taskbar."
    checked: root.valueShowTitle
    onToggled: checked => {
                 root.valueShowTitle = checked;
                 saveSettings();
               }
    enabled: !isVerticalBar
  }

  NTextInput {
    id: titleWidthInput
    visible: root.valueShowTitle && !isVerticalBar
    Layout.fillWidth: true
    label: "Title width"
    description: "Set the width of window titles in the taskbar (in pixels)."
    text: widgetData.titleWidth || widgetMetadata.titleWidth
    placeholderText: "Enter width in pixels"
    onEditingFinished: saveSettings()
  }

  NToggle {
    Layout.fillWidth: true
    visible: !isVerticalBar && root.valueShowTitle
    label: "Smart width"
    description: "Automatically adjust entry width based on the number of entries."
    checked: root.valueSmartWidth
    onToggled: checked => {
                 root.valueSmartWidth = checked;
                 saveSettings();
               }
  }

  NValueSlider {
    visible: root.valueSmartWidth && !isVerticalBar
    Layout.fillWidth: true
    label: "Maximum width"
    description: "Maximum width of the taskbar as a percentage of screen width."
    from: 10
    to: 100
    stepSize: 5
    value: root.valueMaxTaskbarWidth
    onMoved: value => {
               root.valueMaxTaskbarWidth = Math.round(value);
               saveSettings();
             }
    text: Math.round(root.valueMaxTaskbarWidth) + "%"
  }
}
