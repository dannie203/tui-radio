import QtQuick
import Quickshell
import qs.Ui

BarWidget {
  id: root
  moduleName: "aki.hiphop-radio"

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  function launchRadio() {
    Quickshell.execDetached([
      "boombox-toggle"
    ])
  }

  WidgetButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: root.vertical ? "" : "RADIO"
    labelVisible: !root.vertical
    hasVisualContent: true
    tooltipText: "Open Hip-Hop Radio"

    onPressed: function(button) {
      if (button === Qt.LeftButton) root.launchRadio()
    }

    OpticalGlyph {
      anchors.centerIn: parent
      text: "♫"
      color: button.foreground
      fontFamily: button.fontFamily
      fontSize: button.fontSize
      visible: root.vertical
    }
  }
}
