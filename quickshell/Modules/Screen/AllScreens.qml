import QtQuick
import Quickshell
import Quickshell.Wayland

import qs.Common
import qs.Modules.Screen
import qs.Services.UI

// Single screen mode - simplified for single-display setups
// Creates Screen and related components only for the primary screen
Item {
  id: root

  property ShellScreen primaryScreen: Quickshell.screens[0]

  Component.onCompleted: {
    if (primaryScreen) {
      Logger.d("AllScreens", "Single-screen mode - using primary screen:", primaryScreen?.name);
    } else {
      Logger.w("AllScreens", "No primary screen available!");
    }
  }

  // Main Screen - Bar and panels backgrounds
  Screen {
    id: mainScreen
    screen: root.primaryScreen
  }

  // Bar content window
  BarContentWindow {
    id: barContentWindow
    screen: root.primaryScreen
  }

  // BarTriggerZone - thin invisible zone to reveal hidden bar (when auto-hide is enabled)
  BarTriggerZone {
    id: barTriggerZone
    screen: root.primaryScreen
    visible: {
      if (!mainScreen)
        return false;
      if (!BarService.effectivelyVisible)
        return false;
      return Settings.getBarDisplayModeForScreen(root.primaryScreen?.name) === "auto_hide";
    }
  }

  // BarExclusionZone - created after Screen has fully loaded
  // Note: Exclusion zone should NOT be affected by hideOnOverview setting.
  // When bar is hidden during overview, the exclusion zone should remain to prevent
  // windows from moving into the bar area. Auto-hide is handled by the component
  // itself via ExclusionMode.Ignore/Auto.
  Repeater {
    model: Settings.data.bar.barType === "framed" ? ["top", "bottom", "left", "right"] : [Settings.getBarPositionForScreen(root.primaryScreen?.name)]
    delegate: Item {
      required property var modelData
      BarExclusionZone {
        screen: root.primaryScreen
        edge: modelData
      }
    }
  }

  // PopupMenuWindow - reusable popup window for both tray menus and context menus
  PopupMenuWindow {
    id: popupMenuWindow
    screen: root.primaryScreen
  }
}
