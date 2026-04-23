import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.Compositor
import qs.Services.Core
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  ColumnLayout {
    spacing: Style.marginL

    Repeater {
      model: Quickshell.screens || []
      delegate: NBox {
        Layout.fillWidth: true
        implicitHeight: Math.round(contentCol.implicitHeight + Style.margin2L)
        color: Color.mSurface

        property var brightnessMonitor: BrightnessService.getMonitorForScreen(modelData)
        property real localBrightness: 0.5
        property bool localBrightnessChanging: false
        readonly property string automaticOptionLabel: {
          var baseLabel = "Default";
          var autoDevicePath = (BrightnessService.availableBacklightDevices && BrightnessService.availableBacklightDevices.length > 0) ? BrightnessService.availableBacklightDevices[0] : "";
          if (autoDevicePath === "")
            return baseLabel;

          var autoDeviceName = BrightnessService.getBacklightDeviceName(autoDevicePath) || autoDevicePath;
          return baseLabel + "(" + autoDeviceName + ")";
        }
        readonly property var backlightDeviceOptions: {
          var options = [
                {
                  "key": "",
                  "name": automaticOptionLabel
                }
              ];

          var devices = BrightnessService.availableBacklightDevices || [];
          for (var i = 0; i < devices.length; i++) {
            var devicePath = devices[i];
            var deviceName = BrightnessService.getBacklightDeviceName(devicePath) || devicePath;
            options.push({
                           "key": devicePath,
                           "name": deviceName
                         });
          }
          return options;
        }

        onBrightnessMonitorChanged: {
          if (brightnessMonitor && !localBrightnessChanging)
            localBrightness = brightnessMonitor.brightness || 0.5;
        }

        Connections {
          target: BrightnessService
          function onMonitorBrightnessChanged(monitor, newBrightness) {
            if (monitor === brightnessMonitor && !localBrightnessChanging) {
              localBrightness = newBrightness;
            }
          }
        }
        Connections {
          target: brightnessMonitor
          ignoreUnknownSignals: true
          function onBrightnessUpdated() {
            if (brightnessMonitor && !localBrightnessChanging) {
              localBrightness = brightnessMonitor.brightness || 0;
            }
          }
        }
        Timer {
          id: debounceTimer
          interval: 120
          repeat: false
          onTriggered: {
            if (brightnessMonitor && brightnessMonitor.brightnessControlAvailable && Math.abs(localBrightness - brightnessMonitor.brightness) >= 0.005) {
              brightnessMonitor.setBrightness(localBrightness);
            }
          }
        }

        ColumnLayout {
          id: contentCol
          width: parent.width - 2 * Style.marginL
          x: Style.marginL
          y: Style.marginL
          spacing: Style.marginXXS

          RowLayout {
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignBottom

            NText {
              text: modelData.name || "Unknown"
              pointSize: Style.fontSizeL
              font.weight: Style.fontWeightSemiBold
              Layout.alignment: Qt.AlignBottom
            }

            NText {
              Layout.fillWidth: true
              text: {
                const compositorScale = CompositorService.getDisplayScale(modelData.name);
                "{model} ({width}x{height} @ {scale}x)";
              }
              pointSize: Style.fontSizeS
              color: Color.mOnSurfaceVariant
              wrapMode: Text.WordWrap
              horizontalAlignment: Text.AlignRight
              Layout.alignment: Qt.AlignBottom
            }
          }

          ColumnLayout {
            spacing: Style.marginS
            Layout.fillWidth: true
            visible: brightnessMonitor !== undefined && brightnessMonitor !== null

            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginL

              NText {
                text: "Brightness"
                Layout.preferredWidth: 90
                Layout.alignment: Qt.AlignVCenter
              }

              NValueSlider {
                id: brightnessSlider
                from: 0
                to: 1
                value: localBrightness
                stepSize: 0.01
                enabled: brightnessMonitor ? brightnessMonitor.brightnessControlAvailable : false
                onMoved: value => {
                           if (brightnessMonitor && brightnessMonitor.brightnessControlAvailable) {
                             localBrightness = value;
                             debounceTimer.restart();
                           }
                         }
                onPressedChanged: (pressed, value) => {
                                    localBrightnessChanging = pressed;
                                    if (brightnessMonitor && brightnessMonitor.brightnessControlAvailable) {
                                      if (pressed) {
                                        localBrightness = value;
                                        debounceTimer.restart();
                                      } else {
                                        localBrightness = value;
                                        debounceTimer.restart();
                                      }
                                    }
                                  }
                Layout.fillWidth: true
              }

              NText {
                text: brightnessMonitor ? Math.round(localBrightness * 100) + "%" : "N/A"
                Layout.preferredWidth: 55
                horizontalAlignment: Text.AlignRight
                Layout.alignment: Qt.AlignVCenter
                opacity: brightnessMonitor && !brightnessMonitor.brightnessControlAvailable ? 0.5 : 1.0
              }

              Item {
                Layout.preferredWidth: 30
                Layout.fillHeight: true
                NIcon {
                  icon: brightnessMonitor && brightnessMonitor.method == "internal" ? "device-laptop" : "device-desktop"
                  anchors.right: parent.right
                  anchors.verticalCenter: parent.verticalCenter
                  opacity: brightnessMonitor && !brightnessMonitor.brightnessControlAvailable ? 0.5 : 1.0
                }
              }
            }

            NText {
              visible: brightnessMonitor && !brightnessMonitor.brightnessControlAvailable && !(brightnessMonitor.method === "internal" && brightnessMonitor.initInProgress)
              text: !Settings.data.brightness.enableDdcSupport ? "Brightness control unavailable. Enable \"External brightness support\" to control this display's brightness." : "Brightness control is not available for this display."
              pointSize: Style.fontSizeXS
              color: Color.mOnSurfaceVariant
              Layout.fillWidth: true
              wrapMode: Text.WordWrap
            }

            NComboBox {
              Layout.fillWidth: true
              visible: brightnessMonitor && brightnessMonitor.method === "internal"
              label: "Backlight device"
              description: "Select a backlight device for this output."
              model: backlightDeviceOptions
              currentKey: BrightnessService.getMappedBacklightDevice(modelData.name) || ""
              onSelected: key => BrightnessService.setMappedBacklightDevice(modelData.name, key)
            }
          }
        }
      }
    }

    NSpinBox {
      Layout.fillWidth: true
      label: "Brightness step size"
      description: "Adjust the step size for brightness changes (scroll wheel and keyboard shortcuts)."
      minimum: 1
      maximum: 50
      value: Settings.data.brightness.brightnessStep
      stepSize: 1
      suffix: "%"
      onValueChanged: Settings.data.brightness.brightnessStep = value
      defaultValue: Settings.getDefaultValue("brightness.brightnessStep")
    }

    NToggle {
      Layout.fillWidth: true
      label: "Enforce minimum brightness (1%)"
      description: "Solves the problem of backlight completely turning off on some displays at 0% brightness."
      checked: Settings.data.brightness.enforceMinimum
      onToggled: checked => Settings.data.brightness.enforceMinimum = checked
      defaultValue: Settings.getDefaultValue("brightness.enforceMinimum")
    }

    NToggle {
      Layout.fillWidth: true
      label: "External brightness support"
      description: "Enable DDCUtil support for controlling brightness on external displays via DDC/CI protocol."
      checked: Settings.data.brightness.enableDdcSupport
      onToggled: checked => {
                   Settings.data.brightness.enableDdcSupport = checked;
                 }
      defaultValue: Settings.getDefaultValue("brightness.enableDdcSupport")
    }
  }
}
