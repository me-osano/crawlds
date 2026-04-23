import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property list<var> cardsModel: []
  property list<var> cardsDefault: [
    {
      "id": "calendar-header-card",
      "text": "Calendar Header",
      "enabled": true,
      "required": true
    },
    {
      "id": "calendar-month-card",
      "text": "Calendar Month",
      "enabled": true,
      "required": false
    }
  ]

  function saveCards() {
    var toSave = [];
    for (var i = 0; i < cardsModel.length; i++) {
      toSave.push({
                    "id": cardsModel[i].id,
                    "enabled": cardsModel[i].enabled
                  });
    }
    Settings.data.calendar.cards = toSave;
  }

  Component.onCompleted: {
    // Starts empty
    cardsModel = [];

    // Add the cards available in settings
    for (var i = 0; i < Settings.data.calendar.cards.length; i++) {
      const settingCard = Settings.data.calendar.cards[i];

      for (var j = 0; j < cardsDefault.length; j++) {
        if (settingCard.id === cardsDefault[j].id) {
          var card = cardsDefault[j];
          card.enabled = settingCard.enabled;
          cardsModel.push(card);
        }
      }
    }

    // Add any missing cards from default
    for (var i = 0; i < cardsDefault.length; i++) {
      var found = false;
      for (var j = 0; j < cardsModel.length; j++) {
        if (cardsModel[j].id === cardsDefault[i].id) {
          found = true;
          break;
        }
      }

      if (!found) {
        var card = cardsDefault[i];
        cardsModel.push(card);
      }
    }

    saveCards();
  }

  // Clock style section
  ColumnLayout {
    spacing: Style.marginM
    Layout.fillWidth: true

    NToggle {
      label: "Use analog style clock"
      description: "Show an analog style clock on the calendar window."
      checked: Settings.data.location.analogClockInCalendar
      onToggled: checked => Settings.data.location.analogClockInCalendar = checked
    }

    NToggle {
      label: "Show week numbers"
      description: "Display the week of the year (e.g. week 38) in the calendar."
      checked: Settings.data.location.showWeekNumberInCalendar
      onToggled: checked => Settings.data.location.showWeekNumberInCalendar = checked
    }

    NToggle {
      label: "Show calendar events"
      description: "Display events in the calendar panel."
      checked: Settings.data.location.showCalendarEvents
      onToggled: checked => Settings.data.location.showCalendarEvents = checked
    }
  }

  NDivider {
    Layout.fillWidth: true
  }

  // Calendar Cards Management Section
  ColumnLayout {
    spacing: Style.marginXXS
    Layout.fillWidth: true

    NReorderCheckboxes {
      Layout.fillWidth: true
      model: cardsModel
      onItemToggled: function (index, enabled) {
        var newModel = cardsModel.slice();
        newModel[index] = Object.assign({}, newModel[index], {
                                          "enabled": enabled
                                        });
        cardsModel = newModel;
        saveCards();
      }
      onItemsReordered: function (fromIndex, toIndex) {
        var newModel = cardsModel.slice();
        var item = newModel.splice(fromIndex, 1)[0];
        newModel.splice(toIndex, 0, item);
        cardsModel = newModel;
        saveCards();
      }
    }
  }
}
