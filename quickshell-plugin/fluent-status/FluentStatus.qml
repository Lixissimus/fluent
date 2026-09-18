// QtQuick provides the basic QML object types, such as Item and Timer.
import QtQuick
// Controls contains higher-level controls; this draft does not use many yet,
// but keeping the import makes it easy to add controls to the popup later.
import QtQuick.Controls
// Layouts provides ColumnLayout and its Layout.* attached properties.
import QtQuick.Layouts
// Quickshell provides shell-specific objects such as PanelWindow.
import Quickshell
// Quickshell.Io provides Process and StdioCollector for command output.
import Quickshell.Io
// Quickshell.Wayland provides Wayland window types used by the popup.
import Quickshell.Wayland
// qs.Commons and qs.Ui are Omarchy's shared theme and widget components.
import qs.Commons
import qs.Ui

// BarWidget is the base object that the Omarchy bar embeds in one slot.
BarWidget {
  // An id is a local name for this object. It lets child objects refer to it.
  id: root
  // The module name identifies this widget to the bar and to its settings.
  moduleName: "fluent.status"

  // This is the data displayed by the popup. It will come from fluent-status.
  property var instances: []
  // False means the last query failed, for example because the aggregator is off.
  property bool available: false
  // Controls whether the popup window is visible.
  property bool detailsOpen: false

  // readonly properties are calculated values; QML updates them automatically
  // when one of the properties used in the expression changes.
  readonly property int instanceCount: instances.length
  // Use the bar's current theme when loaded by Omarchy, with a fallback for
  // standalone testing or when no bar has been injected yet.
  readonly property color foreground: bar ? bar.foreground : Color.foreground
  readonly property color panelBackground: bar ? bar.background : Color.background

  // The bar uses these implicit dimensions to decide how much space to reserve.
  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  // Start the long-running watcher unless it is already connected. The guard
  // prevents duplicate watcher processes from being started.
  function refresh() {
    if (!statusProcess.running) statusProcess.running = true
  }

  // Convert one JSON line from the watcher into QML data. JSON.parse returns a
  // JS object, so data.instances can be used directly as a Repeater model.
  function updateStatus(raw) {
    try {
      var data = JSON.parse(raw)
      // Use an empty list if the response is valid but omits "instances".
      root.instances = data.instances || []
      root.available = true
    } catch (error) {
      // Invalid JSON is treated like an unavailable service rather than taking
      // down the shell process.
      root.instances = []
      root.available = false
    }
  }

  // Component.onCompleted is a lifecycle signal. This runs once after the
  // widget and its child objects have been constructed.
  Component.onCompleted: refresh()

  // Process runs the external watcher without blocking the Quickshell UI.
  Process {
    id: statusProcess
    // The watcher should stay alive and print one complete JSON document per
    // line whenever the aggregator's instance list changes.
    command: ["fluent-status", "--watch"]

    // SplitParser calls onRead for every newline-delimited stdout message,
    // allowing the icon to react immediately without polling.
    stdout: SplitParser {
      onRead: function(line) {
        root.updateStatus(line)
      }
    }

    // If the watcher exits because the aggregator disconnected or the helper
    // crashed, show the unavailable state and try again after a short delay.
    onExited: function(exitCode) {
      root.instances = []
      root.available = false
      restartTimer.start()
    }
  }

  // A one-shot retry timer handles the case where fluent-status starts before
  // the aggregator. It also prevents a tight restart loop after an error.
  Timer {
    id: restartTimer
    interval: 5000
    repeat: false
    onTriggered: root.refresh()
  }

  // This is the small clickable icon that appears in the bar.
  BarIconButton {
    id: button
    // Fill the slot supplied by the bar, while BarIconButton handles sizing.
    anchors.fill: parent
    bar: root.bar
    // This is a Font Awesome-style glyph from the active icon font.
    text: "\uf085"
    // The button receives its active styling when at least one instance exists.
    active: root.instanceCount > 0
    slotSize: Style.bar.statusSlot
    // The tooltip changes reactively as query results arrive.
    tooltipText: root.available
      ? (root.instanceCount + " Fluent instance" + (root.instanceCount === 1 ? "" : "s"))
      : "Fluent aggregator unavailable"
    // onPressed is a signal handler. Refresh first, then toggle the popup.
    onPressed: {
      root.refresh()
      root.detailsOpen = !root.detailsOpen
    }
  }

  // PanelWindow is a separate Wayland window. It is used here as a deliberately
  // simple popup; a production plugin could use Omarchy's richer Panel base.
  PanelWindow {
    id: detailsWindow
    // Binding visible to detailsOpen means assigning one property controls the
    // lifetime/visibility of the popup from the bar button.
    visible: root.detailsOpen
    // Keep the popup width stable while allowing its height to fit each row.
    implicitWidth: 260
    implicitHeight: Math.max(92, 58 + root.instanceCount * 34)
    color: root.panelBackground
    // Keep the status popup above normal application windows.
    aboveWindows: true

    // Attach the popup to the top-right corner of its layer surface.
    anchors {
      top: true
      right: true
    }

    // Rectangle gives the popup a visible background and border.
    Rectangle {
      anchors.fill: parent
      color: root.panelBackground
      border.color: root.foreground
      border.width: 1

      // ColumnLayout places the header, summary, and instance rows vertically.
      ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        // The popup title is a normal Qt text item.
        Text {
          Layout.fillWidth: true
          text: "Fluent instances"
          color: root.foreground
          font.family: root.bar ? root.bar.fontFamily : Style.font.family
          font.pixelSize: 14
          font.bold: true
        }

        // The summary shows which of the three states we are in: unavailable,
        // available-but-empty, or available with one or more instances.
        Text {
          Layout.fillWidth: true
          text: !root.available
            ? "Aggregator unavailable"
            : root.instanceCount === 0
              ? "No running instances"
              : root.instanceCount + " running"
          color: root.foreground
          font.family: root.bar ? root.bar.fontFamily : Style.font.family
          font.pixelSize: 12
        }

        // Repeater creates one delegate object for each element in the model.
        // Here the model is the instances array returned by fluent-status.
        Repeater {
          model: root.instances

          // The delegate is the visual representation of one instance.
          delegate: Text {
            // modelData is the current array element, such as { pid: 1234 }.
            required property var modelData
            Layout.fillWidth: true
            text: "PID " + modelData.pid
            color: root.foreground
            font.family: root.bar ? root.bar.fontFamily : Style.font.family
            font.pixelSize: 12
          }
        }
      }
    }
  }
}