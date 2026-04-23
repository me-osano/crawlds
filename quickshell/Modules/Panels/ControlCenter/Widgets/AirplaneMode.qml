import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.Core
import qs.Services.UI
import qs.Widgets

NIconButtonHot {
  property ShellScreen screen

  icon: !Settings.data.network.airplaneModeEnabled ? "plane-off" : "plane"
  hot: Settings.data.network.airplaneModeEnabled
  tooltipText: "Airplane Mode"
  onClicked: {
    BluetoothService.setAirplaneMode(!Settings.data.network.airplaneModeEnabled);
  }
}
