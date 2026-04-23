import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  enabled: Settings.data.notifications.enabled

  NToggle {
    label: "Respect expire timeout"
    description: "Use the expire timeout set in the notification."
    checked: Settings.data.notifications.respectExpireTimeout
    onToggled: checked => Settings.data.notifications.respectExpireTimeout = checked
    defaultValue: Settings.getDefaultValue("notifications.respectExpireTimeout")
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Low urgency"
    description: "How long low priority notifications stay visible."
    from: 1
    to: 30
    stepSize: 1
    showReset: true
    value: Settings.data.notifications.lowUrgencyDuration
    onMoved: value => Settings.data.notifications.lowUrgencyDuration = value
    text: Settings.data.notifications.lowUrgencyDuration + "s"
    defaultValue: Settings.getDefaultValue("notifications.lowUrgencyDuration")
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Normal urgency"
    description: "How long normal priority notifications stay visible."
    from: 1
    to: 30
    stepSize: 1
    showReset: true
    value: Settings.data.notifications.normalUrgencyDuration
    onMoved: value => Settings.data.notifications.normalUrgencyDuration = value
    text: Settings.data.notifications.normalUrgencyDuration + "s"
    defaultValue: Settings.getDefaultValue("notifications.normalUrgencyDuration")
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Critical urgency"
    description: "How long critical priority notifications stay visible."
    from: 1
    to: 30
    stepSize: 1
    showReset: true
    value: Settings.data.notifications.criticalUrgencyDuration
    onMoved: value => Settings.data.notifications.criticalUrgencyDuration = value
    text: Settings.data.notifications.criticalUrgencyDuration + "s"
    defaultValue: Settings.getDefaultValue("notifications.criticalUrgencyDuration")
  }
}
