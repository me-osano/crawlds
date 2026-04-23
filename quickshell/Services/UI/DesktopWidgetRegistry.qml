pragma Singleton

import QtQuick
import Quickshell
import qs.Common
import qs.Modules.DesktopWidgets.Widgets

Singleton {
  id: root

  property bool editMode: false

  property Component clockComponent: Component {
    DesktopClock {}
  }
  property Component mediaPlayerComponent: Component {
    DesktopMediaPlayer {}
  }
  property Component systemStatComponent: Component {
    DesktopSystemStat {}
  }

  property var widgets: ({})
  property var widgetSettingsMap: ({
                               "Clock": "WidgetSettings/ClockSettings.qml",
                               "MediaPlayer": "WidgetSettings/MediaPlayerSettings.qml",
                               "SystemStat": "WidgetSettings/SystemStatSettings.qml"
                             })
  property var widgetMetadata: ({
                                "Clock": {
                                  "showBackground": true,
                                  "clockStyle": "digital",
                                  "clockColor": "none",
                                  "useCustomFont": false,
                                  "format": "HH:mm\\nd MMMM yyyy"
                                },
                                "MediaPlayer": {
                                  "showBackground": true,
                                  "visualizerType": "linear",
                                  "hideMode": "visible",
                                  "showButtons": true,
                                  "showAlbumArt": true,
                                  "showVisualizer": true,
                                  "roundedCorners": true
                                },
                                "SystemStat": {
                                  "showBackground": true,
                                  "statType": "CPU",
                                  "diskPath": "/",
                                  "roundedCorners": true,
                                  "layout": "bottom"
                                }
                              })
  property var cpuIntensiveWidgets: ["SystemStat"]
  property var pluginWidgets: ({})
  property var pluginWidgetMetadata: ({})

  Component.onCompleted: {
    var widgetsObj = {};
    widgetsObj["Clock"] = clockComponent;
    widgetsObj["MediaPlayer"] = mediaPlayerComponent;
    widgetsObj["SystemStat"] = systemStatComponent;
    widgets = widgetsObj;
    Logger.i("DesktopWidgetRegistry", "Service started");
  }

  function init() {
    Logger.i("DesktopWidgetRegistry", "Service started");
  }

  function getWidget(id) {
    return widgets[id] || null;
  }

  function hasWidget(id) {
    return id in widgets;
  }

  function getAvailableWidgets() {
    return Object.keys(widgets);
  }

  function widgetHasUserSettings(id) {
    return widgetMetadata[id] !== undefined;
  }

  function isCpuIntensive(id) {
    if (pluginWidgetMetadata[id]?.cpuIntensive)
      return true;
    return cpuIntensiveWidgets.indexOf(id) >= 0;
  }

  function isPluginWidget(id) {
    return id.startsWith("plugin:");
  }

  function getPluginWidgets() {
    return Object.keys(pluginWidgets);
  }

  function getWidgetDisplayName(widgetId) {
    if (widgetId.startsWith("plugin:")) {
      var pluginId = widgetId.replace("plugin:", "");
      return pluginId;
    }
    return widgetId;
  }

  function updateWidgetData(monitorName, widgetIndex, properties) {
    if (widgetIndex < 0 || !monitorName) {
      return;
    }

    var monitorWidgets = Settings.data.desktopWidgets.monitorWidgets || [];
    var newMonitorWidgets = monitorWidgets.slice();

    for (var i = 0; i < newMonitorWidgets.length; i++) {
      if (newMonitorWidgets[i].name === monitorName) {
        var wdgts = (newMonitorWidgets[i].widgets || []).slice();
        if (widgetIndex < wdgts.length) {
          wdgts[widgetIndex] = Object.assign({}, wdgts[widgetIndex], properties);
          newMonitorWidgets[i] = Object.assign({}, newMonitorWidgets[i], {
                                                 "widgets": wdgts
                                               });
          Settings.data.desktopWidgets.monitorWidgets = newMonitorWidgets;
        }
        break;
      }
    }
  }

  property var currentSettingsDialog: null

  function openWidgetSettings(screen, widgetIndex, widgetId, widgetData) {
    if (!widgetId || !screen) {
      return;
    }

    if (root.currentSettingsDialog) {
      root.currentSettingsDialog.close();
      root.currentSettingsDialog.destroy();
      root.currentSettingsDialog = null;
    }

    var hasSettings = root.widgetSettingsMap[widgetId] !== undefined;

    if (!hasSettings) {
      Logger.w("DesktopWidgetRegistry", "Widget does not have settings:", widgetId);
      return;
    }

    var popupMenuWindow = PanelService.getPopupMenuWindow(screen);
    if (!popupMenuWindow) {
      Logger.e("DesktopWidgetRegistry", "No popup menu window found for screen");
      return;
    }

    if (popupMenuWindow.hideDynamicMenu) {
      popupMenuWindow.hideDynamicMenu();
    }

    var component = Qt.createComponent(Quickshell.shellDir + "/Modules/Settings/DesktopWidgets/DesktopWidgetSettingsDialog.qml");

    function instantiateAndOpen() {
      var dialog = component.createObject(popupMenuWindow.dialogParent, {
                                            "widgetIndex": widgetIndex,
                                            "widgetData": widgetData,
                                            "widgetId": widgetId,
                                            "sectionId": screen.name,
                                            "screen": screen
                                          });

      if (dialog) {
        root.currentSettingsDialog = dialog;
        dialog.updateWidgetSettings.connect((sec, idx, settings) => {
                                              root.updateWidgetData(sec, idx, settings);
                                            });
        popupMenuWindow.hasDialog = true;
        dialog.closed.connect(() => {
                                popupMenuWindow.hasDialog = false;
                                popupMenuWindow.close();
                                if (root.currentSettingsDialog === dialog) {
                                  root.currentSettingsDialog = null;
                                }
                                dialog.destroy();
                              });
        dialog.open();
      } else {
        Logger.e("DesktopWidgetRegistry", "Failed to create widget settings dialog");
      }
    }

    if (component.status === Component.Ready) {
      instantiateAndOpen();
    } else if (component.status === Component.Error) {
      Logger.e("DesktopWidgetRegistry", "Error loading settings dialog component:", component.errorString());
    } else {
      component.statusChanged.connect(() => {
                                        if (component.status === Component.Ready) {
                                          instantiateAndOpen();
                                        } else if (component.status === Component.Error) {
                                          Logger.e("DesktopWidgetRegistry", "Error loading settings dialog component:", component.errorString());
                                        }
                                      });
    }
  }
}