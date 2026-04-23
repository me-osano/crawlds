import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.System
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  Layout.fillHeight: true

  ColumnLayout {
    spacing: Style.marginL
    Layout.fillWidth: true

    NComboBox {
      id: controlCenterPosition
      label: "Position"
      description: "Choose where the control center panel appears when opened."
      Layout.fillWidth: true
      model: [
        {
          "key": "close_to_bar_button",
          "name": "Close to bar button"
        },
        {
          "key": "center",
          "name": "Center"
        },
        {
          "key": "top_center",
          "name": "Top center"
        },
        {
          "key": "top_left",
          "name": "Top left"
        },
        {
          "key": "top_right",
          "name": "Top right"
        },
        {
          "key": "bottom_center",
          "name": "Bottom center"
        },
        {
          "key": "bottom_left",
          "name": "Bottom left"
        },
        {
          "key": "bottom_right",
          "name": "Bottom right"
        }
      ]
      currentKey: Settings.data.controlCenter.position
      onSelected: function (key) {
        Settings.data.controlCenter.position = key;
      }
      defaultValue: Settings.getDefaultValue("controlCenter.position")
    }

    NComboBox {
      id: diskPathComboBox
      Layout.fillWidth: true
      label: "System monitor disk path"
      description: "Select which disk mount point the system monitor card in the control center should monitor."
      model: {
        const paths = Object.keys(SystemStatService.diskPercents).sort();
        return paths.map(path => ({
                                    key: path,
                                    name: path
                                  }));
      }
      currentKey: Settings.data.controlCenter.diskPath || "/"
      onSelected: key => Settings.data.controlCenter.diskPath = key
      defaultValue: Settings.getDefaultValue("controlCenter.diskPath") || "/"
    }
  }

  Rectangle {
    Layout.fillHeight: true
  }
}
