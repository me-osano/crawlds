import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services.Core
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property var timeOptions

  signal checkWlsunset

  NToggle {
    label: "Enable Night Light"
    description: "Apply a warm color filter to reduce blue light emission."
    checked: Settings.data.nightLight.enabled
    onToggled: checked => {
                 if (checked) {
                   root.checkWlsunset();
                 } else {
                   Settings.data.nightLight.enabled = false;
                   Settings.data.nightLight.forced = false;
                   NightLightService.apply();
                   ToastService.showNotice("Night Light", "Disabled", "nightlight-off");
                 }
               }
  }

  ColumnLayout {
    enabled: Settings.data.nightLight.enabled
    spacing: Style.marginL
    Layout.fillWidth: true

    NLabel {
      label: "Night"
      description: "Controls the temperature during nighttime."
      Layout.fillWidth: true
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginM

      NSlider {
        id: nightSlider
        Layout.fillWidth: true
        from: 1000
        to: 6500
        value: Settings.data.nightLight.nightTemp

        onValueChanged: {
          var dayTemp = parseInt(Settings.data.nightLight.dayTemp);
          var v = Math.round(value);
          if (!isNaN(dayTemp)) {
            var maxNight = dayTemp - 500;
            v = Math.min(maxNight, Math.max(1000, v));
          } else {
            v = Math.max(1000, v);
          }
          if (v !== value)
            value = v;
        }

        onPressedChanged: {
          if (!pressed) {
            var dayTemp = parseInt(Settings.data.nightLight.dayTemp);
            var v = Math.round(value);
            if (!isNaN(dayTemp)) {
              var maxNight = dayTemp - 500;
              v = Math.min(maxNight, Math.max(1000, v));
            } else {
              v = Math.max(1000, v);
            }
            Settings.data.nightLight.nightTemp = v;
          }
        }
      }

      NText {
        text: nightSlider.value + "K"
        pointSize: Style.fontSizeM
        color: Color.mOnSurfaceVariant
        Layout.alignment: Qt.AlignVCenter
      }
    }

    NLabel {
      label: "Day"
      description: "Controls the temperature during daytime."
      Layout.fillWidth: true
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginM

      NSlider {
        id: daySlider
        Layout.fillWidth: true
        from: 1000
        to: 6500
        value: Settings.data.nightLight.dayTemp

        onValueChanged: {
          var nightTemp = parseInt(Settings.data.nightLight.nightTemp);
          var v = Math.round(value);
          if (!isNaN(nightTemp)) {
            var minDay = nightTemp + 500;
            v = Math.max(minDay, Math.min(6500, v));
          } else {
            v = Math.min(6500, v);
          }
          if (v !== value)
            value = v;
        }

        onPressedChanged: {
          if (!pressed) {
            var nightTemp = parseInt(Settings.data.nightLight.nightTemp);
            var v = Math.round(value);
            if (!isNaN(nightTemp)) {
              var minDay = nightTemp + 500;
              v = Math.max(minDay, Math.min(6500, v));
            } else {
              v = Math.min(6500, v);
            }
            Settings.data.nightLight.dayTemp = v;
          }
        }
      }

      NText {
        text: daySlider.value + "K"
        pointSize: Style.fontSizeM
        color: Color.mOnSurfaceVariant
        Layout.alignment: Qt.AlignVCenter
      }
    }

    NToggle {
      label: "Automatic scheduling"
      description: "Based on the sunset and sunrise time in <i>{location}</i> — recommended."
      checked: Settings.data.nightLight.autoSchedule
      onToggled: checked => Settings.data.nightLight.autoSchedule = checked
    }

    ColumnLayout {
      spacing: Style.marginS
      Layout.fillWidth: true
      visible: !Settings.data.nightLight.autoSchedule && !Settings.data.nightLight.forced

      NLabel {
        label: "Manual scheduling"
        description: "Set custom times for sunrise and sunset."
      }

      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginS

        NText {
          text: "Sunrise time"
          pointSize: Style.fontSizeM
          color: Color.mOnSurfaceVariant
          Layout.alignment: Qt.AlignVCenter
        }

        NComboBox {
          model: root.timeOptions
          currentKey: Settings.data.nightLight.manualSunrise
          placeholder: "Select start time"
          onSelected: key => Settings.data.nightLight.manualSunrise = key
          Layout.fillWidth: true
        }
      }

      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginS

        NText {
          text: "Sunset time"
          pointSize: Style.fontSizeM
          color: Color.mOnSurfaceVariant
          Layout.alignment: Qt.AlignVCenter
        }

        NComboBox {
          model: root.timeOptions
          currentKey: Settings.data.nightLight.manualSunset
          placeholder: "Select stop time"
          onSelected: key => Settings.data.nightLight.manualSunset = key
          Layout.fillWidth: true
        }
      }
    }

    NToggle {
      label: "Force activation"
      description: "Ignores the schedule and applies the night filter immediately."
      checked: Settings.data.nightLight.forced
      onToggled: checked => {
                   Settings.data.nightLight.forced = checked;
                   if (checked && !Settings.data.nightLight.enabled) {
                     root.checkWlsunset();
                   } else {
                     NightLightService.apply();
                   }
                 }
    }
  }
}
