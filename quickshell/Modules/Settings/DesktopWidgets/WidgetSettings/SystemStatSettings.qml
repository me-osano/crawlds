import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.System
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM
  width: 700

  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  property string valueStatType: widgetData.statType !== undefined ? widgetData.statType : widgetMetadata.statType
  property string valueDiskPath: widgetData.diskPath !== undefined ? widgetData.diskPath : widgetMetadata.diskPath
  property bool valueShowBackground: widgetData.showBackground !== undefined ? widgetData.showBackground : widgetMetadata.showBackground
  property bool valueRoundedCorners: widgetData.roundedCorners !== undefined ? widgetData.roundedCorners : (widgetMetadata.roundedCorners !== undefined ? widgetMetadata.roundedCorners : true)
  property string valueLayout: widgetData.layout !== undefined ? widgetData.layout : (widgetMetadata.layout !== undefined ? widgetMetadata.layout : "side")

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.statType = valueStatType;
    settings.diskPath = valueDiskPath;
    settings.showBackground = valueShowBackground;
    settings.roundedCorners = valueRoundedCorners;
    settings.layout = valueLayout;
    settingsChanged(settings);
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Statistic Type"
    description: "Choose which system statistic to display."
    currentKey: valueStatType
    minimumWidth: 260 * Style.uiScaleRatio
    model: {
      let items = [
            {
              "key": "CPU",
              "name": "CPU usage"
            },
            {
              "key": "Memory",
              "name": "Memory"
            },
            {
              "key": "Network",
              "name": "Network traffic"
            },
            {
              "key": "Disk",
              "name": "Disk"
            }
          ];
      if (Settings.data.systemMonitor.enableDgpuMonitoring)
        items.push({
                     "key": "GPU",
                     "name": "GPU temperature"
                   });
      return items;
    }
    onSelected: key => {
                  valueStatType = key;
                  saveSettings();
                }
  }

  NComboBox {
    Layout.fillWidth: true
    visible: valueStatType === "Disk"
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

  NDivider {
    Layout.fillWidth: true
  }

  NToggle {
    Layout.fillWidth: true
    label: "Show background"
    description: "Show the background container for the system stat widget."
    checked: valueShowBackground
    onToggled: checked => {
                 valueShowBackground = checked;
                 saveSettings();
               }
  }

  NToggle {
    Layout.fillWidth: true
    visible: valueShowBackground
    label: "Rounded corners"
    description: "Use rounded corners for the widget background."
    checked: valueRoundedCorners
    onToggled: checked => {
                 valueRoundedCorners = checked;
                 saveSettings();
               }
  }

  NDivider {
    Layout.fillWidth: true
  }

  NComboBox {
    Layout.fillWidth: true
    label: "Layout"
    description: "Choose how the legend is displayed relative to the graph."
    currentKey: valueLayout
    minimumWidth: 260 * Style.uiScaleRatio
    model: [
      {
        "key": "side",
        "name": "Side"
      },
      {
        "key": "bottom",
        "name": "Bottom"
      }
    ]
    onSelected: key => {
                  valueLayout = key;
                  saveSettings();
                }
  }
}
