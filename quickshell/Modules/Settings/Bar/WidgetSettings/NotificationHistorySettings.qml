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
  property bool valueShowUnreadBadge: widgetData.showUnreadBadge !== undefined ? widgetData.showUnreadBadge : widgetMetadata.showUnreadBadge
  property bool valueHideWhenZero: widgetData.hideWhenZero !== undefined ? widgetData.hideWhenZero : widgetMetadata.hideWhenZero
  property bool valueHideWhenZeroUnread: widgetData.hideWhenZeroUnread !== undefined ? widgetData.hideWhenZeroUnread : widgetMetadata.hideWhenZeroUnread
  property string valueUnreadBadgeColor: widgetData.unreadBadgeColor !== undefined ? widgetData.unreadBadgeColor : widgetMetadata.unreadBadgeColor
  property string valueIconColor: widgetData.iconColor !== undefined ? widgetData.iconColor : widgetMetadata.iconColor

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.showUnreadBadge = valueShowUnreadBadge;
    settings.hideWhenZero = valueHideWhenZero;
    settings.hideWhenZeroUnread = valueHideWhenZeroUnread;
    settings.unreadBadgeColor = valueUnreadBadgeColor;
    settings.iconColor = valueIconColor;
    settingsChanged(settings);
  }

  NToggle {
    label: "Show unread badge"
    description: "Display a badge showing the number of unread notifications."
    checked: valueShowUnreadBadge
    onToggled: checked => {
                 valueShowUnreadBadge = checked;
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
    label: "Unread badge color"
    description: "Select the color for the unread notification badge."
    currentKey: valueUnreadBadgeColor
    onSelected: key => {
                  valueUnreadBadgeColor = key;
                  saveSettings();
                }
    visible: valueShowUnreadBadge
  }

  NToggle {
    label: "Hide icon when no notifications"
    description: "Hide the notification icon when there are no notifications."
    checked: valueHideWhenZero
    onToggled: checked => {
                 valueHideWhenZero = checked;
                 saveSettings();
               }
    visible: !valueHideWhenZeroUnread
  }

  NToggle {
    label: "Hide icon when no unread notifications"
    description: "Hide the notification icon when there are no unread notifications."
    checked: valueHideWhenZeroUnread
    onToggled: checked => {
                 valueHideWhenZeroUnread = checked;
                 saveSettings();
               }
  }
}
