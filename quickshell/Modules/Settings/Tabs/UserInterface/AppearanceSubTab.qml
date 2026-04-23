import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NToggle {
    label: "Show tooltips"
    description: "Enable or disable tooltips throughout the interface."
    checked: Settings.data.ui.tooltipsEnabled
    defaultValue: Settings.getDefaultValue("ui.tooltipsEnabled")
    onToggled: checked => Settings.data.ui.tooltipsEnabled = checked
  }

  NToggle {
    label: "Container outline"
    description: "Display an outline around content areas."
    checked: Settings.data.ui.boxBorderEnabled
    defaultValue: Settings.getDefaultValue("ui.boxBorderEnabled")
    onToggled: checked => Settings.data.ui.boxBorderEnabled = checked
  }

  NToggle {
    label: "Drop shadows"
    description: "Enables drop shadows under bars and panels."
    checked: Settings.data.general.enableShadows
    defaultValue: Settings.getDefaultValue("general.enableShadows")
    onToggled: checked => Settings.data.general.enableShadows = checked
  }

  NComboBox {
    visible: Settings.data.general.enableShadows
    label: "Shadow direction"
    description: "Choose where the shadow is cast to."
    Layout.fillWidth: true

    readonly property var shadowOptionsMap: ({
                                               "top_left": {
                                                 "name": "Top left",
                                                 "p": Qt.point(-2, -2)
                                               },
                                               "top": {
                                                 "name": "Top",
                                                 "p": Qt.point(0, -3)
                                               },
                                               "top_right": {
                                                 "name": "Top right",
                                                 "p": Qt.point(2, -2)
                                               },
                                               "left": {
                                                 "name": "Left",
                                                 "p": Qt.point(-3, 0)
                                               },
                                               "center": {
                                                 "name": "Center",
                                                 "p": Qt.point(0, 0)
                                               },
                                               "right": {
                                                 "name": "Right",
                                                 "p": Qt.point(3, 0)
                                               },
                                               "bottom_left": {
                                                 "name": "Bottom left",
                                                 "p": Qt.point(-2, 2)
                                               },
                                               "bottom": {
                                                 "name": "Bottom",
                                                 "p": Qt.point(0, 3)
                                               },
                                               "bottom_right": {
                                                 "name": "Bottom right",
                                                 "p": Qt.point(2, 3)
                                               }
                                             })

    model: Object.keys(shadowOptionsMap).map(function (k) {
      return {
        "key": k,
        "name": shadowOptionsMap[k].name
      };
    })

    currentKey: Settings.data.general.shadowDirection
    defaultValue: Settings.getDefaultValue("general.shadowDirection")

    onSelected: function (key) {
      var opt = shadowOptionsMap[key];
      if (opt) {
        Settings.data.general.shadowDirection = key;
        Settings.data.general.shadowOffsetX = opt.p.x;
        Settings.data.general.shadowOffsetY = opt.p.y;
      }
    }
  }

  NDivider {
    Layout.fillWidth: true
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Interface scaling"
    description: "Changes the size of the general user interface, excluding the bar."
    from: 0.8
    to: 1.2
    stepSize: 0.05
    showReset: true
    value: Settings.data.general.scaleRatio
    defaultValue: Settings.getDefaultValue("general.scaleRatio")
    onMoved: value => Settings.data.general.scaleRatio = value
    text: Math.floor(Settings.data.general.scaleRatio * 100) + "%"
  }

  NDivider {
    Layout.fillWidth: true
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Container radius"
    description: "Adjusts the corner roundness of major layout sections, such as sidebars, cards, and content panels."
    from: 0
    to: 2
    stepSize: 0.01
    showReset: true
    value: Settings.data.general.radiusRatio
    defaultValue: Settings.getDefaultValue("general.radiusRatio")
    onMoved: value => Settings.data.general.radiusRatio = value
    text: Math.floor(Settings.data.general.radiusRatio * 100) + "%"
  }

  NValueSlider {
    Layout.fillWidth: true
    label: "Input radius"
    description: "Controls the curvature of interactive elements, including buttons, toggles, and text fields."
    from: 0
    to: 2
    stepSize: 0.01
    showReset: true
    value: Settings.data.general.iRadiusRatio
    defaultValue: Settings.getDefaultValue("general.iRadiusRatio")
    onMoved: value => Settings.data.general.iRadiusRatio = value
    text: Math.floor(Settings.data.general.iRadiusRatio * 100) + "%"
  }

  NDivider {
    Layout.fillWidth: true
  }

  ColumnLayout {
    spacing: Style.marginL
    Layout.fillWidth: true

    NToggle {
      label: "Disable UI animations"
      description: "Disable all animations for a faster, more responsive experience."
      checked: Settings.data.general.animationDisabled
      defaultValue: Settings.getDefaultValue("general.animationDisabled")
      onToggled: checked => Settings.data.general.animationDisabled = checked
    }

    ColumnLayout {
      spacing: Style.marginXXS
      Layout.fillWidth: true
      visible: !Settings.data.general.animationDisabled

      RowLayout {
        spacing: Style.marginL
        Layout.fillWidth: true

        NValueSlider {
          Layout.fillWidth: true
          label: "Animation speed"
          description: "Adjust global animation speed."
          from: 0
          to: 2.0
          stepSize: 0.01
          value: Settings.data.general.animationSpeed
          defaultValue: Settings.getDefaultValue("general.animationSpeed")
          onMoved: value => Settings.data.general.animationSpeed = Math.max(value, 0.05)
          text: Math.round(Settings.data.general.animationSpeed * 100) + "%"
        }

        Item {
          Layout.preferredWidth: 30 * Style.uiScaleRatio
          Layout.preferredHeight: 30 * Style.uiScaleRatio

          NIconButton {
            icon: "restore"
            baseSize: Style.baseWidgetSize * 0.8
            tooltipText: "Reset animation speed"
            onClicked: Settings.data.general.animationSpeed = Settings.getDefaultValue("general.animationSpeed")
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
          }
        }
      }
    }
  }
}
