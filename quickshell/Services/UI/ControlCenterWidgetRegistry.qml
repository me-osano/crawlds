pragma Singleton

import QtQuick
import Quickshell
import qs.Common
import qs.Modules.Panels.ControlCenter.Widgets

Singleton {
  id: root

  // Widget registry object mapping widget names to components
  property var widgets: ({
                           "AirplaneMode": airplaneModeComponent,
                           "Bluetooth": bluetoothComponent,
                           "CustomButton": customButtonComponent,
                           "DarkMode": darkModeComponent,
                           "NightLight": nightLightComponent,
                           "Notifications": notificationsComponent,
                           "PowerProfile": powerProfileComponent,
                           "WiFi": networkComponent,
                           "Network": networkComponent,
                           "WallpaperSelector": wallpaperSelectorComponent
                         })

  property var widgetMetadata: ({
                                  "CustomButton": {
                                    "icon": "heart",
                                    "onClicked": "",
                                    "onRightClicked": "",
                                    "onMiddleClicked": "",
                                    "stateChecksJson": "[]",
                                    "generalTooltipText": "",
                                    "enableOnStateLogic": false,
                                    "showExecTooltip": true
                                  }
                                })

  property var cpuIntensiveWidgets: ["SystemStat"]

  // Component definitions - these are loaded once at startup
  property Component airplaneModeComponent: Component {
    AirplaneMode {}
  }
  property Component bluetoothComponent: Component {
    Bluetooth {}
  }
  property Component customButtonComponent: Component {
    CustomButton {}
  }
  property Component darkModeComponent: Component {
    DarkMode {}
  }
  property Component nightLightComponent: Component {
    NightLight {}
  }
  property Component notificationsComponent: Component {
    Notifications {}
  }
  property Component powerProfileComponent: Component {
    PowerProfile {}
  }
  property Component networkComponent: Component {
    Network {}
  }
  property Component wallpaperSelectorComponent: Component {
    WallpaperSelector {}
  }

  function init() {
    Logger.i("ControlCenterWidgetRegistry", "Service started");
  }

  // ------------------------------
  // Helper function to get widget component by name
  function getWidget(id) {
    return widgets[id] || null;
  }

  // Helper function to check if widget exists
  function hasWidget(id) {
    return id in widgets;
  }

  // Get list of available widget id
  function getAvailableWidgets() {
    return Object.keys(widgets);
  }

  // Helper function to check if widget has user settings
  function widgetHasUserSettings(id) {
    return widgetMetadata[id] !== undefined;
  }

  function isCpuIntensive(id) {
    return false;
    return cpuIntensiveWidgets.indexOf(id) >= 0;
  }

  // Check if a widget is a plugin widget
  function isPluginWidget(id) {
    return id.startsWith("plugin:");
  }
}
