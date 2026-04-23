pragma Singleton

import QtQuick
import Quickshell
import qs.Common

Singleton {
  id: root

  property bool initComplete: false
  property bool nextDarkModeState: false

  Connections {
    target: Settings.data.colorSchemes
    enabled: Settings.data.colorSchemes.schedulingMode == "manual"
    function onManualSunriseChanged() {
      const changes = root.collectManualChanges();
      root.applyCurrentMode(changes);
      root.scheduleNextMode(changes);
    }
    function onManualSunsetChanged() {
      const changes = root.collectManualChanges();
      root.applyCurrentMode(changes);
      root.scheduleNextMode(changes);
    }
  }

  Connections {
    target: Settings.data.colorSchemes
    function onSchedulingModeChanged() {
      root.update();
    }
  }

  Connections {
    target: Time
    function onResumed() {
      Logger.i("DarkModeService", "System resumed - re-evaluating dark mode");
      root.update();
      resumeRetryTimer.restart();
    }
  }

  Timer {
    id: timer
    onTriggered: {
      Settings.data.colorSchemes.darkMode = root.nextDarkModeState;
      root.update();
    }
  }

  Timer {
    id: resumeRetryTimer
    interval: 2000
    repeat: false
    onTriggered: {
      Logger.i("DarkModeService", "Resume retry - re-evaluating dark mode again");
      root.update();
    }
  }

  function init() {
    Logger.i("DarkModeService", "Service started");
    root.update();
  }

  function update() {
    if (Settings.data.colorSchemes.schedulingMode == "manual") {
      const changes = collectManualChanges();
      initComplete = true;
      applyCurrentMode(changes);
      scheduleNextMode(changes);
    }
  }

  function parseTime(timeString) {
    const parts = timeString.split(":").map(Number);
    return {
      "hour": parts[0],
      "minute": parts[1]
    };
  }

  function collectManualChanges() {
    const sunriseTime = parseTime(Settings.data.colorSchemes.manualSunrise);
    const sunsetTime = parseTime(Settings.data.colorSchemes.manualSunset);

    const now = new Date();
    const year = now.getFullYear();
    const month = now.getMonth();
    const day = now.getDate();

    const yesterdaysSunset = new Date(year, month, day - 1, sunsetTime.hour, sunsetTime.minute);
    const todaysSunrise = new Date(year, month, day, sunriseTime.hour, sunriseTime.minute);
    const todaysSunset = new Date(year, month, day, sunsetTime.hour, sunsetTime.minute);
    const tomorrowsSunrise = new Date(year, month, day + 1, sunriseTime.hour, sunriseTime.minute);

    return [
          {
            "time": yesterdaysSunset.getTime(),
            "darkMode": true
          },
          {
            "time": todaysSunrise.getTime(),
            "darkMode": false
          },
          {
            "time": todaysSunset.getTime(),
            "darkMode": true
          },
          {
            "time": tomorrowsSunrise.getTime(),
            "darkMode": false
          }
        ];
  }

  function applyCurrentMode(changes) {
    const now = Date.now();
    Logger.i("DarkModeService", `Applying mode at ${new Date(now).toLocaleString()} (${now})`);

    // changes.findLast(change => change.time < now) // not available in QML...
    let lastChange = null;
    for (var i = 0; i < changes.length; i++) {
      Logger.d("DarkModeService", `Checking change: time=${changes[i].time} (${new Date(changes[i].time).toLocaleString()}), darkMode=${changes[i].darkMode}`);
      if (changes[i].time < now) {
        lastChange = changes[i];
      }
    }

    if (lastChange) {
      Logger.i("DarkModeService", `Selected change: time=${lastChange.time}, darkMode=${lastChange.darkMode}`);
      Settings.data.colorSchemes.darkMode = lastChange.darkMode;
      Logger.d("DarkModeService", `Reset: darkmode=${lastChange.darkMode}`);
    } else {
      Logger.w("DarkModeService", "No suitable change found for current time!");
    }
  }

  function scheduleNextMode(changes) {
    const now = Date.now();
    const nextChange = changes.find(change => change.time > now);
    if (nextChange) {
      root.nextDarkModeState = nextChange.darkMode;
      timer.interval = nextChange.time - now;
      timer.restart();
      Logger.d("DarkModeService", `Scheduled: darkmode=${nextChange.darkMode} in ${timer.interval} ms`);
    }
  }
}
