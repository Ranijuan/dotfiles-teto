import QtQuick
import QtQuick.Layouts
import Caelestia
import Caelestia.Config
import qs.components
import qs.services
import qs.modules.lock.center

RowLayout {
    id: root

    required property var lock
    readonly property bool isNight: {
        const h = new Date().getHours();
        return h >= 18 || h < 6;
    }
    readonly property string wdesc: (Weather.description ?? "").toLowerCase()
    readonly property bool rainy: /rain|drizzle|storm|thunder|fog|mist|haze|overcast/.test(wdesc)
    readonly property bool hasNotifs: Notifs.list.length > 0
    readonly property real clockScale: Math.min(0.6, (lock.screen?.height ?? 1440) / 1440)
    readonly property string bosque: {
        if (isNight) return rainy ? "noche-lluvia.jpg" : "noche-clara.jpg";
        return rainy ? "dia-lluvia.jpg" : "dia-claro.jpg";
    }

    spacing: Tokens.spacing.medium

    // Columna izquierda: clima con bosque, y la hora sobre el bosque
    Item {
        Layout.preferredWidth: root.width * 0.24
        Layout.maximumWidth: root.width * 0.24
        Layout.fillHeight: true
        clip: true

        Image {
            anchors.fill: parent
            source: "file:/home/ranijuan/.config/caelestia/weather-bg/" + root.bosque
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
        }

        Rectangle {
            anchors.fill: parent
            color: root.isNight ? "#55061024" : "#22000000"
        }

        WeatherInfo {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            rootHeight: root.height
        }

        ColumnLayout {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: Tokens.padding.large
            spacing: Tokens.spacing.small

            Clock {
                Layout.alignment: Qt.AlignHCenter
                centerScale: root.clockScale
            }

            StyledText {
                Layout.alignment: Qt.AlignHCenter
                text: Time.format("dddd • d MMM").toUpperCase()
                color: Colours.palette.m3onSurface
                font: Tokens.font.title.builders.medium.weight(Font.DemiBold).build()
            }
        }
    }

    // Columna central: arriba Arch Teto y candado; abajo reproductor con letra
    ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        spacing: Tokens.spacing.medium

        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: root.height * 0.38
            spacing: Tokens.spacing.medium

            Fetch {
                Layout.fillWidth: true
                Layout.preferredHeight: root.height * 0.18
                Layout.maximumHeight: root.height * 0.18
                rootHeight: root.height
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: Tokens.spacing.medium

                ProfilePic {
                    Layout.alignment: Qt.AlignHCenter
                    centerWidth: root.width * 0.2
                }

                PasswordInput {
                    Layout.alignment: Qt.AlignHCenter
                    centerScale: Math.max(0.8, root.clockScale)
                    centerWidth: root.width * 0.2
                    lock: root.lock
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Media {
                anchors.fill: parent
                lock: root.lock
            }

            Lyrics {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: parent.height * 0.5
            }
        }
    }

    // Columna derecha: graficas arriba y notificaciones abajo (solo si hay)
    ColumnLayout {
        Layout.preferredWidth: root.width * 0.2
        Layout.maximumWidth: root.width * 0.2
        Layout.fillHeight: true
        spacing: Tokens.spacing.medium

        Resources {
            Layout.fillWidth: true
        }

        StyledRect {
            visible: root.hasNotifs
            Layout.fillWidth: true
            Layout.fillHeight: true

            bottomRightRadius: Tokens.rounding.extraLarge
            radius: Tokens.rounding.medium
            color: Colours.tPalette.m3surfaceContainer

            NotifDock {
                lock: root.lock
            }
        }

        Item {
            visible: !root.hasNotifs
            Layout.fillHeight: true
        }
    }
}
