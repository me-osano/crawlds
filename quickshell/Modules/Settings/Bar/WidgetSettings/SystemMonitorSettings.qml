import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.System
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM

  // Properties to receive data from parent
  property var screen: null
  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  readonly property string barPosition: Settings.getBarPositionForScreen(screen?.name)
  readonly property bool isVerticalBar: barPosition === "left" || barPosition === "right"

  // Local, editable state for checkboxes
  property bool valueCompactMode: widgetData.compactMode !== undefined ? widgetData.compactMode : widgetMetadata.compactMode
  property string valueIconColor: widgetData.iconColor !== undefined ? widgetData.iconColor : widgetMetadata.iconColor
  property string valueTextColor: widgetData.textColor !== undefined ? widgetData.textColor : widgetMetadata.textColor
  property bool valueUseMonospaceFont: widgetData.useMonospaceFont !== undefined ? widgetData.useMonospaceFont : widgetMetadata.useMonospaceFont
  property bool valueUsePadding: widgetData.usePadding !== undefined ? widgetData.usePadding : widgetMetadata.usePadding
  property bool valueShowCpuUsage: widgetData.showCpuUsage !== undefined ? widgetData.showCpuUsage : widgetMetadata.showCpuUsage
  property bool valueShowCpuFreq: widgetData.showCpuFreq !== undefined ? widgetData.showCpuFreq : widgetMetadata.showCpuFreq
  property bool valueShowCpuTemp: widgetData.showCpuTemp !== undefined ? widgetData.showCpuTemp : widgetMetadata.showCpuTemp
  property bool valueShowGpuTemp: widgetData.showGpuTemp !== undefined ? widgetData.showGpuTemp : widgetMetadata.showGpuTemp
  property bool valueShowLoadAverage: widgetData.showLoadAverage !== undefined ? widgetData.showLoadAverage : widgetMetadata.showLoadAverage
  property bool valueShowMemoryUsage: widgetData.showMemoryUsage !== undefined ? widgetData.showMemoryUsage : widgetMetadata.showMemoryUsage
  property bool valueShowMemoryAsPercent: widgetData.showMemoryAsPercent !== undefined ? widgetData.showMemoryAsPercent : widgetMetadata.showMemoryAsPercent
  property bool valueShowSwapUsage: widgetData.showSwapUsage !== undefined ? widgetData.showSwapUsage : widgetMetadata.showSwapUsage
  property bool valueShowNetworkStats: widgetData.showNetworkStats !== undefined ? widgetData.showNetworkStats : widgetMetadata.showNetworkStats
  property bool valueShowDiskUsage: widgetData.showDiskUsage !== undefined ? widgetData.showDiskUsage : widgetMetadata.showDiskUsage
  property bool valueShowDiskUsageAsPercent: widgetData.showDiskUsageAsPercent !== undefined ? widgetData.showDiskUsageAsPercent : widgetMetadata.showDiskUsageAsPercent
  property bool valueShowDiskAvailable: widgetData.showDiskAvailable !== undefined ? widgetData.showDiskAvailable : widgetMetadata.showDiskAvailable
  property string valueDiskPath: widgetData.diskPath !== undefined ? widgetData.diskPath : widgetMetadata.diskPath

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.compactMode = valueCompactMode;
    settings.iconColor = valueIconColor;
    settings.textColor = valueTextColor;
    settings.useMonospaceFont = valueUseMonospaceFont;
    settings.usePadding = valueUsePadding;
    settings.showCpuUsage = valueShowCpuUsage;
    settings.showCpuFreq = valueShowCpuFreq;
    settings.showCpuTemp = valueShowCpuTemp;
    settings.showGpuTemp = valueShowGpuTemp;
    settings.showLoadAverage = valueShowLoadAverage;
    settings.showMemoryUsage = valueShowMemoryUsage;
    settings.showMemoryAsPercent = valueShowMemoryAsPercent;
    settings.showSwapUsage = valueShowSwapUsage;
    settings.showNetworkStats = valueShowNetworkStats;
    settings.showDiskUsage = valueShowDiskUsage;
    settings.showDiskUsageAsPercent = valueShowDiskUsageAsPercent;
    settings.showDiskAvailable = valueShowDiskAvailable;
    settings.diskPath = valueDiskPath;

    settingsChanged(settings);
  }

  NToggle {
    Layout.fillWidth: true
    label: "Compact mode"
    description: "Display stats as mini bar charts instead of text values. Prevents layout shifting."
    checked: valueCompactMode
    onToggled: checked => {
                 valueCompactMode = checked;
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
    visible: !valueCompactMode
  }

  NToggle {
    Layout.fillWidth: true
    label: "Monospace font"
    description: "Use monospace font for consistent character width."
    checked: valueUseMonospaceFont
    onToggled: checked => {
                 valueUseMonospaceFont = checked;
                 saveSettings();
               }
    visible: !valueCompactMode
  }

  NToggle {
    Layout.fillWidth: true
    label: "Pad text"
    description: isVerticalBar ? "Vertical taskbar does not support padding text." : !valueUseMonospaceFont ? "A monospace font is needed for this feature." : "Pads the text values with leading spaces to prevent layout shifting."
    checked: valueUsePadding && !isVerticalBar && valueUseMonospaceFont
    onToggled: checked => {
                 valueUsePadding = checked;
                 saveSettings();
               }
    visible: !valueCompactMode
    enabled: !isVerticalBar && valueUseMonospaceFont
  }

  NDivider {
    Layout.fillWidth: true
  }

  NToggle {
    id: showCpuUsage
    Layout.fillWidth: true
    label: "CPU usage"
    description: "Display current CPU usage percentage."
    checked: valueShowCpuUsage
    onToggled: checked => {
                 valueShowCpuUsage = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showCpuFreq
    Layout.fillWidth: true
    label: "Show CPU frequency"
    description: "Display the current CPU clock speed in GHz."
    checked: valueShowCpuFreq
    onToggled: checked => {
                 valueShowCpuFreq = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showCpuTemp
    Layout.fillWidth: true
    label: "CPU temperature"
    description: "Show CPU temperature readings if available."
    checked: valueShowCpuTemp
    onToggled: checked => {
                 valueShowCpuTemp = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showLoadAverage
    Layout.fillWidth: true
    label: "Load average"
    description: "Display system load average."
    checked: valueShowLoadAverage
    onToggled: checked => {
                 valueShowLoadAverage = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showGpuTemp
    Layout.fillWidth: true
    label: "GPU temperature"
    description: "Show GPU temperature readings if available."
    checked: valueShowGpuTemp
    onToggled: checked => {
                 valueShowGpuTemp = checked;
                 saveSettings();
               }
    visible: SystemStatService.gpuAvailable
  }

  NToggle {
    id: showMemoryUsage
    Layout.fillWidth: true
    label: "Memory usage"
    description: "Display current RAM usage information."
    checked: valueShowMemoryUsage
    onToggled: checked => {
                 valueShowMemoryUsage = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showMemoryAsPercent
    Layout.fillWidth: true
    label: "Memory as percentage"
    description: "Show memory usage as a percentage instead of absolute values."
    checked: valueShowMemoryAsPercent
    onToggled: checked => {
                 valueShowMemoryAsPercent = checked;
                 saveSettings();
               }
    visible: valueShowMemoryUsage
  }

  NToggle {
    id: showSwapUsage
    Layout.fillWidth: true
    label: "Swap usage"
    description: "Show swap memory usage."
    checked: valueShowSwapUsage
    onToggled: checked => {
                 valueShowSwapUsage = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showNetworkStats
    Layout.fillWidth: true
    label: "Network traffic"
    description: "Display network upload and download speeds."
    checked: valueShowNetworkStats
    onToggled: checked => {
                 valueShowNetworkStats = checked;
                 saveSettings();
               }
  }

  NDivider {
    Layout.fillWidth: true
  }

  NToggle {
    id: showDiskUsage
    Layout.fillWidth: true
    label: "Storage usage"
    description: "Show disk space usage information."
    checked: valueShowDiskUsage
    onToggled: checked => {
                 valueShowDiskUsage = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showDiskUsageAsPercent
    Layout.fillWidth: true
    label: "Disk as percentage"
    description: "Show disk space as percentage instead of absolute values."
    checked: valueShowDiskUsageAsPercent
    onToggled: checked => {
                 valueShowDiskUsageAsPercent = checked;
                 saveSettings();
               }
  }

  NToggle {
    id: showDiskAvailable
    Layout.fillWidth: true
    label: "Disk space available"
    description: "Shows how much disk space is available instead of how much is used."
    checked: valueShowDiskAvailable
    onToggled: checked => {
                 valueShowDiskAvailable = checked;
                 saveSettings();
               }
  }

  NComboBox {
    id: diskPathComboBox
    Layout.fillWidth: true
    label: "Disk path"
    description: "Select which disk mount point to monitor."
    model: {
      const paths = Object.keys(SystemStatService.diskPercents).sort();
      return paths.map(path => ({
                                  key: path,
                                  name: path
                                }));
    }
    currentKey: valueDiskPath
    onSelected: key => {
                  valueDiskPath = key;
                  saveSettings();
                }
  }
}
