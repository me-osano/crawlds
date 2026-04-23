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

  property string valueLabelMode: widgetData.labelMode !== undefined ? widgetData.labelMode : widgetMetadata.labelMode
  property bool valueHideUnoccupied: widgetData.hideUnoccupied !== undefined ? widgetData.hideUnoccupied : widgetMetadata.hideUnoccupied
  property bool valueFollowFocusedScreen: widgetData.followFocusedScreen !== undefined ? widgetData.followFocusedScreen : widgetMetadata.followFocusedScreen
  property int valueCharacterCount: widgetData.characterCount !== undefined ? widgetData.characterCount : widgetMetadata.characterCount

  // Grouped mode settings
  property bool valueShowApplications: widgetData.showApplications !== undefined ? widgetData.showApplications : widgetMetadata.showApplications
  property bool valueShowLabelsOnlyWhenOccupied: widgetData.showLabelsOnlyWhenOccupied !== undefined ? widgetData.showLabelsOnlyWhenOccupied : widgetMetadata.showLabelsOnlyWhenOccupied
  property bool valueColorizeIcons: widgetData.colorizeIcons !== undefined ? widgetData.colorizeIcons : widgetMetadata.colorizeIcons
  property real valueUnfocusedIconsOpacity: widgetData.unfocusedIconsOpacity !== undefined ? widgetData.unfocusedIconsOpacity : widgetMetadata.unfocusedIconsOpacity
  property real valueGroupedBorderOpacity: widgetData.groupedBorderOpacity !== undefined ? widgetData.groupedBorderOpacity : widgetMetadata.groupedBorderOpacity
  property bool valueEnableScrollWheel: widgetData.enableScrollWheel !== undefined ? widgetData.enableScrollWheel : widgetMetadata.enableScrollWheel
  property real valueIconScale: widgetData.iconScale !== undefined ? widgetData.iconScale : widgetMetadata.iconScale
  property string valueFocusedColor: widgetData.focusedColor !== undefined ? widgetData.focusedColor : widgetMetadata.focusedColor
  property string valueOccupiedColor: widgetData.occupiedColor !== undefined ? widgetData.occupiedColor : widgetMetadata.occupiedColor
  property string valueEmptyColor: widgetData.emptyColor !== undefined ? widgetData.emptyColor : widgetMetadata.emptyColor
  property bool valueShowBadge: widgetData.showBadge !== undefined ? widgetData.showBadge : widgetMetadata.showBadge
  property real valuePillSize: widgetData.pillSize !== undefined ? widgetData.pillSize : widgetMetadata.pillSize

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.labelMode = valueLabelMode;
    settings.hideUnoccupied = valueHideUnoccupied;
    settings.characterCount = valueCharacterCount;
    settings.followFocusedScreen = valueFollowFocusedScreen;
    settings.showApplications = valueShowApplications;
    settings.showLabelsOnlyWhenOccupied = valueShowLabelsOnlyWhenOccupied;
    settings.colorizeIcons = valueColorizeIcons;
    settings.unfocusedIconsOpacity = valueUnfocusedIconsOpacity;
    settings.groupedBorderOpacity = valueGroupedBorderOpacity;
    settings.enableScrollWheel = valueEnableScrollWheel;
    settings.iconScale = valueIconScale;
    settings.focusedColor = valueFocusedColor;
    settings.occupiedColor = valueOccupiedColor;
    settings.emptyColor = valueEmptyColor;
    settings.showBadge = valueShowBadge;
    settings.pillSize = valuePillSize;
    settingsChanged(settings);
  }

  NComboBox {
    id: labelModeCombo
    label: "Label mode"
    description: "Choose how workspace labels are displayed."
    model: [
      {
        "key": "none",
        "name": "None"
      },
      {
        "key": "index",
        "name": "Index"
      },
      {
        "key": "name",
        "name": "Name"
      },
      {
        "key": "index+name",
        "name": "Index and name"
      }
    ]
    currentKey: widgetData.labelMode || widgetMetadata.labelMode
    onSelected: key => {
                  valueLabelMode = key;
                  saveSettings();
                }
    minimumWidth: 200
  }

  NSpinBox {
    label: "Character count"
    description: "Number of characters to display from workspace names (1-10)."
    from: 1
    to: 10
    value: valueCharacterCount
    onValueChanged: {
      valueCharacterCount = value;
      saveSettings();
    }
    visible: valueLabelMode === "name"
  }

  NValueSlider {
    label: "Pill size"
    description: "Adjust the size of workspace pills."
    from: 0.4
    to: 1.0
    stepSize: 0.01
    value: valuePillSize
    defaultValue: widgetMetadata.pillSize
    showReset: true
    onMoved: value => {
               valuePillSize = value;
               saveSettings();
             }
    text: Math.round(valuePillSize * 100) + "%"
    visible: !valueShowApplications
  }

  NToggle {
    label: "Hide unoccupied"
    description: "Don't display workspaces without windows."
    checked: valueHideUnoccupied
    onToggled: checked => {
                 valueHideUnoccupied = checked;
                 saveSettings();
               }
  }

  NToggle {
    label: "Show labels only when occupied"
    description: "Only show workspace labels when they contain windows."
    checked: valueShowLabelsOnlyWhenOccupied
    onToggled: checked => {
                 valueShowLabelsOnlyWhenOccupied = checked;
                 saveSettings();
               }
  }

  NToggle {
    label: "Follow focused screen"
    description: "Display workspaces from the currently focused screen, rather than the screen where the bar is located."
    checked: valueFollowFocusedScreen
    onToggled: checked => {
                 valueFollowFocusedScreen = checked;
                 saveSettings();
               }
  }

  NToggle {
    label: "Scroll to switch workspaces"
    description: "Switch between workspaces using the mouse scroll wheel."
    checked: valueEnableScrollWheel
    onToggled: checked => {
                 valueEnableScrollWheel = checked;
                 saveSettings();
               }
  }

  NDivider {
    Layout.fillWidth: true
  }

  NToggle {
    label: "Show applications"
    description: "Display application icons inside each workspace."
    checked: valueShowApplications
    onToggled: checked => {
                 valueShowApplications = checked;
                 saveSettings();
               }
  }

  NToggle {
    label: "Show workspace badge"
    description: "Show the workspace number badge in grouped mode."
    checked: valueShowBadge
    onToggled: checked => {
                 valueShowBadge = checked;
                 saveSettings();
               }
    visible: valueShowApplications
  }

  NToggle {
    label: "Colorize icons"
    description: "Apply theme colors to active window icon."
    checked: valueColorizeIcons
    onToggled: checked => {
                 valueColorizeIcons = checked;
                 saveSettings();
               }
    visible: valueShowApplications
  }

  NValueSlider {
    label: "Unfocused icons opacity"
    description: "Set the opacity level for unfocused app icons."
    from: 0
    to: 1
    stepSize: 0.01
    value: valueUnfocusedIconsOpacity
    onMoved: value => {
               valueUnfocusedIconsOpacity = value;
               saveSettings();
             }
    text: Math.floor(valueUnfocusedIconsOpacity * 100) + "%"
    visible: valueShowApplications
  }

  NValueSlider {
    label: "Border opacity"
    description: "Set the opacity level for workspace container borders."
    from: 0
    to: 1
    stepSize: 0.01
    value: valueGroupedBorderOpacity
    onMoved: value => {
               valueGroupedBorderOpacity = value;
               saveSettings();
             }
    text: Math.floor(valueGroupedBorderOpacity * 100) + "%"
    visible: valueShowApplications
  }

  NValueSlider {
    label: "Icon scaling"
    description: "Sets the scaling factor for taskbar icons."
    from: 0.5
    to: 1
    stepSize: 0.01
    value: valueIconScale
    onMoved: value => {
               valueIconScale = value;
               saveSettings();
             }
    text: Math.round(valueIconScale * 100) + "%"
    visible: valueShowApplications
  }

  NDivider {
    Layout.fillWidth: true
  }

  NColorChoice {
    label: "Focused workspace color"
    description: "Set the background color for the focused workspace."
    currentKey: valueFocusedColor
    onSelected: key => {
                  valueFocusedColor = key;
                  saveSettings();
                }
  }

  NColorChoice {
    label: "Occupied workspace color"
    description: "Set the background color for occupied workspaces."
    currentKey: valueOccupiedColor
    onSelected: key => {
                  valueOccupiedColor = key;
                  saveSettings();
                }
  }

  NColorChoice {
    label: "Empty workspace color"
    description: "Set the background color for empty workspaces."
    currentKey: valueEmptyColor
    onSelected: key => {
                  valueEmptyColor = key;
                  saveSettings();
                }
  }
}
