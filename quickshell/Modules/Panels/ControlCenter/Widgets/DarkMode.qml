import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.Core
import qs.Widgets

NIconButtonHot {
  property ShellScreen screen

  icon: "dark-mode"
  tooltipText: Settings.data.colorSchemes.darkMode ? "Light Mode" : "Dark Mode"
  onClicked: Settings.data.colorSchemes.darkMode = !Settings.data.colorSchemes.darkMode
}
