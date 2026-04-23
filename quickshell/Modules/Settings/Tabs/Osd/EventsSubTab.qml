import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Modules.OSD
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property var addType
  property var removeType

  Repeater {
    model: [
      {
        type: OSD.Type.Volume,
        label: "Volume",
        description: "Show on volume changes"
      },
      {
        type: OSD.Type.InputVolume,
        label: "Input Volume",
        description: "Show on input volume changes"
      },
      {
        type: OSD.Type.Brightness,
        label: "Brightness",
        description: "Show on brightness changes"
      },
      {
        type: OSD.Type.LockKey,
        label: "Lock Key",
        description: "Show when caps lock or num lock is toggled"
      }
    ]
    delegate: NCheckbox {
      required property var modelData
      Layout.fillWidth: true

      label: modelData.label
      description: modelData.description
      checked: (Settings.data.osd.enabledTypes || []).includes(modelData.type)
      onToggled: checked => {
                   if (checked) {
                     Settings.data.osd.enabledTypes = root.addType(Settings.data.osd.enabledTypes, modelData.type);
                   } else {
                     Settings.data.osd.enabledTypes = root.removeType(Settings.data.osd.enabledTypes, modelData.type);
                   }
                 }
    }
  }
}
