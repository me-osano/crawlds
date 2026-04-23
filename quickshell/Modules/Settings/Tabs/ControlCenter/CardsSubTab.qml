import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services.System
import qs.Services.UI
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  Layout.fillHeight: true

  required property var cardsModel
  required property list<var> cardsDefault

  function saveCards() {
    var toSave = [];
    for (var i = 0; i < cardsModel.length; i++) {
      toSave.push({
        "id": cardsModel[i].id,
        "enabled": cardsModel[i].enabled
      });
    }
    Settings.data.controlCenter.cards = toSave;
  }

    NText {
      text: "Customize which controls appear in the control center and in what order."
      wrapMode: Text.WordWrap
      Layout.fillWidth: true
    }

    ColumnLayout {
      spacing: Style.marginXXS
      Layout.fillWidth: true

      NReorderCheckboxes {
        Layout.fillWidth: true
        model: root.cardsModel
        onItemToggled: function (index, enabled) {
          var newModel = root.cardsModel.slice();
          newModel[index] = Object.assign({}, newModel[index], {
                                            "enabled": enabled
                                          });
          root.cardsModel = newModel;
          saveCards();
        }
        onItemsReordered: function (fromIndex, toIndex) {
          var newModel = root.cardsModel.slice();
          var item = newModel.splice(fromIndex, 1)[0];
          newModel.splice(toIndex, 0, item);
          root.cardsModel = newModel;
          saveCards();
        }
      }
    }
  }
