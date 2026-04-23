import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  NToggle {
    label: "Use 12-hour time format"
    description: "Display time in 12-hour format on the lock screen and calendar. The bar clock has its own settings."
    checked: Settings.data.location.use12hourFormat
    onToggled: checked => Settings.data.location.use12hourFormat = checked
  }

  NComboBox {
    label: "First day of week"
    description: "Choose which day starts the week in the calendar."
    currentKey: Settings.data.location.firstDayOfWeek.toString()
    minimumWidth: 260 * Style.uiScaleRatio
    model: [
      {
        "key": "-1",
        "name": "Automatic (use system locale)"
      },
      {
        "key": "6",
        "name": Qt.locale("en").dayName(6, Locale.LongFormat).trim()
      } // Saturday
      ,
      {
        "key": "0",
        "name": Qt.locale("en").dayName(0, Locale.LongFormat).trim()
      } // Sunday
      ,
      {
        "key": "1",
        "name": Qt.locale("en").dayName(1, Locale.LongFormat).trim()
      } // Monday
    ]
    onSelected: key => Settings.data.location.firstDayOfWeek = parseInt(key)
  }
}
