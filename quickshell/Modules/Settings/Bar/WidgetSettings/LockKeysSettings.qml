import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM

  // Properties to receive data from parent
  property var screen: null
  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  // Local state
  property bool valueShowCapsLock: widgetData.showCapsLock !== undefined ? widgetData.showCapsLock : widgetMetadata.showCapsLock
  property bool valueShowNumLock: widgetData.showNumLock !== undefined ? widgetData.showNumLock : widgetMetadata.showNumLock
  property bool valueShowScrollLock: widgetData.showScrollLock !== undefined ? widgetData.showScrollLock : widgetMetadata.showScrollLock

  property string capsIcon: widgetData.capsLockIcon !== undefined ? widgetData.capsLockIcon : widgetMetadata.capsLockIcon
  property string numIcon: widgetData.numLockIcon !== undefined ? widgetData.numLockIcon : widgetMetadata.numLockIcon
  property string scrollIcon: widgetData.scrollLockIcon !== undefined ? widgetData.scrollLockIcon : widgetMetadata.scrollLockIcon

  property bool valueHideWhenOff: widgetData.hideWhenOff !== undefined ? widgetData.hideWhenOff : (widgetMetadata.hideWhenOff !== undefined ? widgetMetadata.hideWhenOff : false)

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.showCapsLock = valueShowCapsLock;
    settings.showNumLock = valueShowNumLock;
    settings.showScrollLock = valueShowScrollLock;
    settings.capsLockIcon = capsIcon;
    settings.numLockIcon = numIcon;
    settings.scrollLockIcon = scrollIcon;
    settings.hideWhenOff = valueHideWhenOff;
    settingsChanged(settings);
  }

  RowLayout {
    spacing: Style.marginM

    NToggle {
      label: "Caps Lock"
      description: "Display Caps Lock status."
      checked: valueShowCapsLock
      onToggled: checked => {
                   valueShowCapsLock = checked;
                   saveSettings();
                 }
    }

    NIcon {
      Layout.alignment: Qt.AlignVCenter
      icon: capsIcon
      pointSize: Style.fontSizeXL
      visible: capsIcon !== ""
    }

    NButton {
      text: "Browse"
      onClicked: capsPicker.open()
      enabled: valueShowCapsLock
    }
  }

  NIconPicker {
    id: capsPicker
    initialIcon: capsIcon
    query: "letter-c"
    onIconSelected: function (iconName) {
      capsIcon = iconName;
      saveSettings();
    }
  }

  RowLayout {
    spacing: Style.marginM

    NToggle {
      label: "Num Lock"
      description: "Display Num Lock status."
      checked: valueShowNumLock
      onToggled: checked => {
                   valueShowNumLock = checked;
                   saveSettings();
                 }
    }

    NIcon {
      Layout.alignment: Qt.AlignVCenter
      icon: numIcon
      pointSize: Style.fontSizeXL
      visible: numIcon !== ""
    }

    NButton {
      text: "Browse"
      onClicked: numPicker.open()
      enabled: valueShowNumLock
    }
  }

  NIconPicker {
    id: numPicker
    initialIcon: numIcon
    query: "letter-n"
    onIconSelected: function (iconName) {
      numIcon = iconName;
      saveSettings();
    }
  }

  RowLayout {
    spacing: Style.marginM

    NToggle {
      label: "Scroll Lock"
      description: "Display Scroll Lock status."
      checked: valueShowScrollLock
      onToggled: checked => {
                   valueShowScrollLock = checked;
                   saveSettings();
                 }
    }

    NIcon {
      Layout.alignment: Qt.AlignVCenter
      icon: scrollIcon
      pointSize: Style.fontSizeXL
      visible: scrollIcon !== ""
    }

    NButton {
      text: "Browse"
      onClicked: scrollPicker.open()
      enabled: valueShowScrollLock
    }
  }

  NIconPicker {
    id: scrollPicker
    initialIcon: scrollIcon
    query: "letter-s"
    onIconSelected: function (iconName) {
      scrollIcon = iconName;
      saveSettings();
    }
  }

  NDivider {
    Layout.fillWidth: true
  }

  NToggle {
    Layout.fillWidth: true
    label: "Hide when off"
    description: "Hide the indicator when the key is not active."
    checked: valueHideWhenOff
    onToggled: checked => {
                 valueHideWhenOff = checked;
                 saveSettings();
               }
  }
}
