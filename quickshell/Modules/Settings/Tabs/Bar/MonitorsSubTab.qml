import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import "../../Bar" as BarSettings
import qs.Common
import qs.Services.Compositor
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property var addMonitor
  property var removeMonitor

  NText {
    text: "Configure which monitors display the bar and customize settings per monitor."
    wrapMode: Text.WordWrap
    Layout.fillWidth: true
  }

  // Monitor cards
  Repeater {
    model: Quickshell.screens || []
    delegate: NBox {
      id: monitorCard
      Layout.fillWidth: true
      implicitHeight: cardContent.implicitHeight + Style.margin2L
      color: Color.mSurface

      required property var modelData
      readonly property string screenName: modelData.name || "Unknown"
      readonly property bool barEnabled: (Settings.data.bar.monitors || []).indexOf(screenName) !== -1
      readonly property bool hasOverride: Settings.hasScreenOverride(screenName)

      // Track if override is enabled (controls both visibility AND whether overrides are applied)
      readonly property bool overrideEnabled: Settings.isScreenOverrideEnabled(screenName)

      // Get effective values for this screen
      readonly property string effectivePosition: Settings.getBarPositionForScreen(screenName)
      readonly property string effectiveDensity: Settings.getBarDensityForScreen(screenName)

      ColumnLayout {
        id: cardContent
        anchors.fill: parent
        anchors.margins: Style.marginL
        spacing: Style.marginM

        RowLayout {
          Layout.fillWidth: true

          // Header: Monitor name and specs
          ColumnLayout {
            Layout.fillWidth: true
            spacing: Style.marginXXS

            NText {
              Layout.fillWidth: true
              text: monitorCard.screenName
              pointSize: Style.fontSizeM
              font.weight: Style.fontWeightBold
              color: Color.mOnSurface
            }

            NText {
              text: {
                const compositorScale = CompositorService.getDisplayScale(monitorCard.screenName);
                return "{model} ({width}x{height} @ {scale}x)";
              }
              pointSize: Style.fontSizeS
              color: Color.mOnSurfaceVariant
            }
          }

          // Enable bar toggle
          NToggle {
            Layout.fillWidth: true
            checked: monitorCard.barEnabled
            onToggled: checked => {
                         if (checked) {
                           Settings.data.bar.monitors = root.addMonitor(Settings.data.bar.monitors, monitorCard.screenName);
                         } else {
                           Settings.data.bar.monitors = root.removeMonitor(Settings.data.bar.monitors, monitorCard.screenName);
                         }
                       }
          }
        }

        NDivider {
          Layout.fillWidth: true
          visible: Settings.data.bar.monitors.includes(monitorCard.screenName)
        }

        // Override section (only visible when bar is enabled)
        ColumnLayout {
          Layout.fillWidth: true
          spacing: Style.marginS
          visible: monitorCard.barEnabled

          // Override toggle
          NToggle {
            Layout.fillWidth: true
            label: "Override global settings"
            description: "Use custom settings for this monitor."
            checked: monitorCard.overrideEnabled
            onToggled: checked => {
                         Settings.setScreenOverride(monitorCard.screenName, "enabled", checked);
                         BarService.widgetsRevision++;
                       }
          }

          // Override controls (only visible when override toggle is on)
          ColumnLayout {
            Layout.fillWidth: true
            spacing: Style.marginS
            visible: monitorCard.overrideEnabled

            // Position override
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS

              NComboBox {
                Layout.fillWidth: true
                label: "Bar position"
                description: "Choose where to place the bar on the screen."
                model: [
                  {
                    "key": "top",
                    "name": "Top"
                  },
                  {
                    "key": "bottom",
                    "name": "Bottom"
                  },
                  {
                    "key": "left",
                    "name": "Left"
                  },
                  {
                    "key": "right",
                    "name": "Right"
                  }
                ]
                currentKey: monitorCard.effectivePosition
                onSelected: key => Settings.setScreenOverride(monitorCard.screenName, "position", key)
              }
            }

            // Density override
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS

              NComboBox {
                Layout.fillWidth: true
                label: "Bar density"
                description: "Adjust the bar's padding for a compact or spacious look."
                model: [
                  {
                    "key": "mini",
                    "name": "Mini"
                  },
                  {
                    "key": "compact",
                    "name": "Compact"
                  },
                  {
                    "key": "default",
                    "name": "Default"
                  },
                  {
                    "key": "comfortable",
                    "name": "Comfortable"
                  },
                  {
                    "key": "spacious",
                    "name": "Spacious"
                  }
                ]
                currentKey: monitorCard.effectiveDensity
                onSelected: key => Settings.setScreenOverride(monitorCard.screenName, "density", key)
              }
            }

            // DisplayMode override
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS

              NComboBox {
                Layout.fillWidth: true
                label: "Display mode"
                description: "Choose when the bar is visible."
                model: [
                  {
                    "key": "always_visible",
                    "name": "Always visible"
                  },
                  {
                    "key": "non_exclusive",
                    "name": "Non-exclusive"
                  },
                  {
                    "key": "auto_hide",
                    "name": "Auto-hide"
                  }
                ]
                currentKey: Settings.getBarDisplayModeForScreen(monitorCard.screenName)
                onSelected: key => Settings.setScreenOverride(monitorCard.screenName, "displayMode", key)
              }
            }

            // Widgets configuration button and Reset all
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS

              NButton {
                id: widgetConfigButton
                property bool expanded: false
                Layout.fillWidth: true
                fontSize: Style.fontSizeS
                text: "Configure widgets"
                icon: expanded ? "chevron-up" : "layout-grid"
                onClicked: expanded = !expanded
              }

              NButton {
                visible: Settings.hasScreenOverride(monitorCard.screenName, "widgets")
                Layout.fillWidth: true
                fontSize: Style.fontSizeS
                text: "Use global widgets"
                icon: "refresh"
                onClicked: {
                  Settings.clearScreenOverride(monitorCard.screenName, "widgets");
                  BarService.widgetsRevision++;
                }
              }

              NButton {
                Layout.fillWidth: true
                fontSize: Style.fontSizeS
                text: "Reset all"
                icon: "restore"
                onClicked: {
                  Settings.clearScreenOverride(monitorCard.screenName);
                  BarService.widgetsRevision++;
                }
              }
            }

            // Inline widget configuration
            BarSettings.MonitorWidgetsConfig {
              visible: widgetConfigButton.expanded
              screen: monitorCard.modelData
              Layout.fillWidth: true
              Layout.topMargin: Style.marginS
            }
          }
        }
      }
    }
  }
}
