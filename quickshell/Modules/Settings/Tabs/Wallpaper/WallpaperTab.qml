import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: 0

  property var screen

  function openMainFolderPicker() {
    mainFolderPicker.open();
  }

  function openMonitorFolderPicker(monitorName) {
    specificFolderMonitorName = monitorName;
    monitorFolderPicker.open();
  }

  property string specificFolderMonitorName: ""

  NTabBar {
    id: subTabBar
    Layout.fillWidth: true
    Layout.bottomMargin: Style.marginM
    distributeEvenly: true
    currentIndex: tabView.currentIndex

    NTabButton {
      text: "General"
      tabIndex: 0
      checked: subTabBar.currentIndex === 0
    }
    NTabButton {
      text: "Look"
      tabIndex: 1
      checked: subTabBar.currentIndex === 1
    }
    NTabButton {
      text: "Automation"
      tabIndex: 2
      checked: subTabBar.currentIndex === 2
    }
  }

  Item {
    Layout.fillWidth: true
    Layout.preferredHeight: Style.marginL
  }

  NTabView {
    id: tabView
    currentIndex: subTabBar.currentIndex

    GeneralSubTab {
      screen: root.screen
      onOpenMainFolderPicker: root.openMainFolderPicker()
      onOpenMonitorFolderPicker: monitorName => root.openMonitorFolderPicker(monitorName)
    }
    LookAndFeelSubTab {
      screen: root.screen
    }
    AutomationSubTab {}
  }

  NFilePicker {
    id: mainFolderPicker
    selectionMode: "folders"
    title: "Select wallpaper folder"
    initialPath: Settings.data.wallpaper.directory || Quickshell.env("HOME") + "/Pictures"
    onAccepted: paths => {
                  if (paths.length > 0) {
                    Settings.data.wallpaper.directory = paths[0];
                  }
                }
  }

  NFilePicker {
    id: monitorFolderPicker
    selectionMode: "folders"
    title: "Select monitor wallpaper folder"
    initialPath: WallpaperService.getMonitorDirectory(specificFolderMonitorName) || Quickshell.env("HOME") + "/Pictures"
    onAccepted: paths => {
                  if (paths.length > 0) {
                    WallpaperService.setMonitorDirectory(specificFolderMonitorName, paths[0]);
                  }
                }
  }
}
