import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  enabled: Settings.data.wallpaper.enabled

  NToggle {
    label: "Scheduled change"
    description: "Automatically change wallpapers at regular intervals."
    checked: Settings.data.wallpaper.automationEnabled
    onToggled: checked => Settings.data.wallpaper.automationEnabled = checked
  }

  ColumnLayout {
    enabled: Settings.data.wallpaper.automationEnabled
    spacing: Style.marginL
    Layout.fillWidth: true

    NComboBox {

      label: "Change mode"
      description: "Choose how wallpapers are selected when changing automatically."
      Layout.fillWidth: true
      model: [
        {
          "key": "random",
          "name": "Random"
        },
        {
          "key": "alphabetical",
          "name": "Alphabetical"
        }
      ]
      currentKey: Settings.data.wallpaper.wallpaperChangeMode || "random"
      onSelected: key => Settings.data.wallpaper.wallpaperChangeMode = key
      defaultValue: Settings.getDefaultValue("wallpaper.transitionType")
    }

    RowLayout {
      NLabel {
        label: "Wallpaper interval"
        description: "How often to change wallpapers automatically."
        Layout.fillWidth: true
      }

      NText {
        text: Time.formatVagueHumanReadableDuration(Settings.data.wallpaper.randomIntervalSec)
        Layout.alignment: Qt.AlignBottom | Qt.AlignRight
      }
    }

    RowLayout {
      id: presetRow
      spacing: Style.marginS
      opacity: enabled ? 1.0 : 0.6

      property var intervalPresets: [5 * 60, 10 * 60, 15 * 60, 30 * 60, 45 * 60, 60 * 60, 90 * 60, 120 * 60]
      property bool isCurrentPreset: {
        return intervalPresets.some(seconds => seconds === Settings.data.wallpaper.randomIntervalSec);
      }
      property bool customForcedVisible: false

      function setIntervalSeconds(sec) {
        Settings.data.wallpaper.randomIntervalSec = sec;
        WallpaperService.restartRandomWallpaperTimer();
        customForcedVisible = false;
      }

      function isSelected(sec) {
        return Settings.data.wallpaper.randomIntervalSec === sec;
      }

      Repeater {
        model: presetRow.intervalPresets
        delegate: IntervalPresetChip {
          seconds: modelData
          label: Time.formatVagueHumanReadableDuration(modelData)
          selected: presetRow.isSelected(modelData) && !customRow.visible
          onClicked: presetRow.setIntervalSeconds(modelData)
        }
      }

      IntervalPresetChip {
        label: customRow.visible ? "Custom" : "Custom…"
        selected: customRow.visible
        onClicked: presetRow.customForcedVisible = !presetRow.customForcedVisible
      }
    }

    RowLayout {
      id: customRow

      visible: presetRow.customForcedVisible || !presetRow.isCurrentPreset
      spacing: Style.marginS
      opacity: enabled ? 1.0 : 0.6
      Layout.topMargin: Style.marginS

      NTextInput {
        label: "Custom interval"
        description: "Enter time as HH:MM (e.g. 01:30)."
        text: {
          const s = Settings.data.wallpaper.randomIntervalSec;
          const h = Math.floor(s / 3600);
          const m = Math.floor((s % 3600) / 60);
          return h + ":" + (m < 10 ? ("0" + m) : m);
        }
        onEditingFinished: {
          const m = text.trim().match(/^(\d{1,2}):(\d{2})$/);
          if (m) {
            let h = parseInt(m[1]);
            let min = parseInt(m[2]);
            if (isNaN(h) || isNaN(min))
              return;
            h = Math.max(0, Math.min(24, h));
            min = Math.max(0, Math.min(59, min));
            Settings.data.wallpaper.randomIntervalSec = (h * 3600) + (min * 60);
            WallpaperService.restartRandomWallpaperTimer();
            presetRow.customForcedVisible = true;
          }
        }
      }
    }
  }

  component IntervalPresetChip: Rectangle {
    property int seconds: 0
    property string label: ""
    property bool selected: false
    signal clicked

    radius: height * 0.5
    color: selected ? Color.mPrimary : Color.mSurfaceVariant
    implicitHeight: Math.max(Style.baseWidgetSize * 0.55, 24)
    implicitWidth: chipLabel.implicitWidth + Style.marginM * 1.5
    border.width: Style.borderS
    border.color: selected ? "transparent" : Color.mOutline

    MouseArea {
      anchors.fill: parent
      cursorShape: Qt.PointingHandCursor
      onClicked: parent.clicked()
    }

    NText {
      id: chipLabel
      anchors.centerIn: parent
      text: parent.label
      pointSize: Style.fontSizeS
      color: parent.selected ? Color.mOnPrimary : Color.mOnSurface
    }
  }
}
