import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Widgets

// Calendar header with date, month/year, and clock
Rectangle {
  id: root
  Layout.fillWidth: true
  Layout.minimumHeight: (60 * Style.uiScaleRatio) + Style.margin2M
  Layout.preferredHeight: (60 * Style.uiScaleRatio) + Style.margin2M
  implicitHeight: (60 * Style.uiScaleRatio) + Style.margin2M
  radius: Style.radiusL
  color: Color.mPrimary

  // Internal state
  readonly property var now: Time.now

  // Expose current month/year for potential synchronization with CalendarMonthCard
  readonly property int currentMonth: now.getMonth()
  readonly property int currentYear: now.getFullYear()

  ColumnLayout {
    id: capsuleColumn
    anchors.top: parent.top
    anchors.left: parent.left
    anchors.bottom: parent.bottom
    anchors.topMargin: Style.marginM
    anchors.bottomMargin: Style.marginM
    anchors.rightMargin: clockLoader.width + Style.margin2XL
    anchors.leftMargin: Style.marginXL
    spacing: 0

    // Combined layout for date, month year
    RowLayout {
      Layout.fillWidth: true
      height: 60 * Style.uiScaleRatio
      clip: true
      spacing: Style.marginS

      // Today day number
      NText {
        Layout.preferredWidth: implicitWidth
        elide: Text.ElideNone
        clip: true
        Layout.alignment: Qt.AlignVCenter | Qt.AlignLeft
        text: root.now.getDate()
        pointSize: Style.fontSizeXXXL * 1.5
        font.weight: Style.fontWeightBold
        color: Color.mOnPrimary
      }

      // Month, year
      ColumnLayout {
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignVCenter | Qt.AlignLeft
        Layout.bottomMargin: Style.marginXXS
        Layout.topMargin: -Style.marginXXS
        spacing: -Style.marginXS

        RowLayout {
          spacing: Style.marginS

          NText {
            text: Qt.locale("en").monthName(root.currentMonth, Locale.LongFormat).toUpperCase()
            pointSize: Style.fontSizeXL * 1.1
            font.weight: Style.fontWeightBold
            color: Color.mOnPrimary
            Layout.alignment: Qt.AlignBaseline
            elide: Text.ElideRight
          }

          NText {
            text: `${root.currentYear}`
            pointSize: Style.fontSizeM
            font.weight: Style.fontWeightBold
            color: Qt.alpha(Color.mOnPrimary, 0.7)
            Layout.alignment: Qt.AlignBaseline
          }
        }
      }
    }
  }

  // Clock display
  Loader {
    id: clockLoader
    anchors.right: parent.right
    anchors.verticalCenter: parent.verticalCenter
    anchors.rightMargin: Style.marginXL
    active: Settings.data.general.clockStyle === "custom"
    sourceComponent: NText {
      text: root.now.toLocaleString(Qt.locale("en"), Settings.data.general.clockFormat)
      pointSize: Style.fontSizeXXXL
      font.weight: Style.fontWeightBold
      color: Color.mOnPrimary
      Layout.alignment: Qt.AlignVCenter | Qt.AlignRight
    }
  }

  readonly property string clockStyle: Settings.data.general.clockStyle
  readonly property string clockFormat: Settings.data.general.clockFormat

  onClockStyleChanged: {
    if (clockStyle === "custom") {
      clockLoader.active = true;
    } else if (clockStyle === "analog") {
      clockLoader.active = false;
    }
  }
}