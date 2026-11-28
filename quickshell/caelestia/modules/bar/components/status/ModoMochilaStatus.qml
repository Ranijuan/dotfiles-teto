import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components

Item {
    id: root

    required property color colour
    required property int parentSpacing

    property bool active: false

    property real iconHeight: active ? icon.implicitHeight : 0
    property real gap: active ? parentSpacing : 0

    implicitWidth: icon.implicitWidth
    implicitHeight: Math.round(iconHeight)

    Behavior on iconHeight {
        Anim {
            type: Anim.SlowEffects
        }
    }

    Behavior on gap {
        Anim {
            type: Anim.SlowEffects
        }
    }

    MaterialIcon {
        id: icon

        anchors.centerIn: parent

        scale: root.active ? 1 : 0.5
        opacity: root.active ? 1 : 0

        text: "backpack"
        color: root.colour
        fill: 1
        grade: 25

        Behavior on opacity {
            Anim {
                type: Anim.DefaultEffects
            }
        }

        Behavior on scale {
            Anim {}
        }
    }

    Process {
        id: check

        command: ["grep", "-q", "^HandleLidSwitch=ignore", "/etc/systemd/logind.conf"]
        onExited: exitCode => root.active = (exitCode === 0)
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (!check.running)
                check.running = true;
        }
    }
}
