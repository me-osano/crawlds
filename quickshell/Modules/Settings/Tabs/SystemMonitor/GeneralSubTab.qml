import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property var screen

  NToggle {
    Layout.fillWidth: true
    Layout.topMargin: Style.marginM
    label: "Enable discrete GPU monitoring"
    description: "Warning: This will wake up your discrete GPU (NVIDIA/AMD), which may significantly impact battery life on laptops with hybrid graphics."
    checked: Settings.data.systemMonitor.enableDgpuMonitoring
    defaultValue: Settings.getDefaultValue("systemMonitor.enableDgpuMonitoring")
    onToggled: checked => Settings.data.systemMonitor.enableDgpuMonitoring = checked
  }

  // Colors Section
  RowLayout {
    Layout.fillWidth: true
    spacing: Style.marginM

    NToggle {
      label: "Use custom highlight colors"
      description: "When disabled, theme default highlight colors are used."
      checked: Settings.data.systemMonitor.useCustomColors
      defaultValue: Settings.getDefaultValue("systemMonitor.useCustomColors")
      onToggled: checked => {
                   // If enabling custom colors and no custom color is saved, persist current theme colors
                   if (checked) {
                     if (!Settings.data.systemMonitor.warningColor || Settings.data.systemMonitor.warningColor === "") {
                       Settings.data.systemMonitor.warningColor = Color.mTertiary.toString();
                     }
                     if (!Settings.data.systemMonitor.criticalColor || Settings.data.systemMonitor.criticalColor === "") {
                       Settings.data.systemMonitor.criticalColor = Color.mError.toString();
                     }
                   }
                   Settings.data.systemMonitor.useCustomColors = checked;
                 }
    }
  }

  RowLayout {
    Layout.fillWidth: true
    spacing: Style.marginXL
    visible: Settings.data.systemMonitor.useCustomColors

    ColumnLayout {
      Layout.fillWidth: true
      spacing: Style.marginM

      NText {
        text: "Warning color"
        pointSize: Style.fontSizeM
      }

      NColorPicker {
        screen: root.screen
        Layout.preferredWidth: Style.sliderWidth
        Layout.preferredHeight: Style.baseWidgetSize
        enabled: Settings.data.systemMonitor.useCustomColors
        selectedColor: Settings.data.systemMonitor.warningColor || Color.mTertiary
        onColorSelected: color => Settings.data.systemMonitor.warningColor = color
      }
    }

    ColumnLayout {
      Layout.fillWidth: true
      spacing: Style.marginM

      NText {
        text: "Critical color"
        pointSize: Style.fontSizeM
      }

      NColorPicker {
        screen: root.screen
        Layout.preferredWidth: Style.sliderWidth
        Layout.preferredHeight: Style.baseWidgetSize
        enabled: Settings.data.systemMonitor.useCustomColors
        selectedColor: Settings.data.systemMonitor.criticalColor || Color.mError
        onColorSelected: color => Settings.data.systemMonitor.criticalColor = color
      }
    }
  }

  NDivider {
    Layout.fillWidth: true
  }

  NTextInput {
    label: "External system monitor command"
    description: "Enter the command or application path to launch when activating the external system monitor application."
    placeholderText: "resources || missioncenter || jdsystemmonitor || corestats || system-monitoring-center || gnome-system-monitor || plasma-systemmonitor || mate-system-monitor || ukui-system-monitor || deepin-system-monitor || pantheon-system-monitor"
    text: Settings.data.systemMonitor.externalMonitor
    defaultValue: Settings.getDefaultValue("systemMonitor.externalMonitor")
    onTextChanged: Settings.data.systemMonitor.externalMonitor = text
  }
}
