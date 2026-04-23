import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import qs.Common
import qs.Modules.Cards
import qs.Modules.Screen
import qs.Services.UI
import qs.Widgets

SmartPanel {
  id: root

  panelContent: Item {
    id: panelContent
    anchors.fill: parent

    readonly property real contentPreferredWidth: Math.round((Settings.data.location.showWeekNumberInCalendar ? 440 : 420) * Style.uiScaleRatio)
    readonly property real contentPreferredHeight: content.implicitHeight + Style.margin2L

    ColumnLayout {
      id: content
      x: Style.marginL
      y: Style.marginL
      width: parent.width - Style.margin2L
      spacing: Style.marginL

      // All clock panel cards
      Repeater {
        model: Settings.data.calendar.cards
        Loader {
          active: modelData.enabled
          visible: active
          Layout.fillWidth: true
          sourceComponent: {
            switch (modelData.id) {
            case "calendar-header-card":
              return calendarHeaderCard;
            case "calendar-month-card":
              return calendarMonthCard;
            default:
              return null;
            }
          }
        }
      }
    }
  }

  Component {
    id: calendarHeaderCard
    CalendarHeaderCard {
      Layout.fillWidth: true
    }
  }

  Component {
    id: calendarMonthCard
    CalendarMonthCard {
      Layout.fillWidth: true
    }
  }
}
