import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.Common
import qs.Services.Core

/**
* IdleFadeOverlay — full-screen fade-to-black shown before each idle action.
*
* A single Loader wraps a Variants so per-screen windows only exist while
* a fade is in progress, keeping VRAM usage at zero at rest.
*
* Any mouse movement cancels the fade and unloads the windows immediately.
*/
Item {
  id: root

  // IdleService removed
}
