pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import QtQuick.Shapes
import Quickshell.Services.UPower
import Caelestia
import Caelestia.Config
import qs.components
import qs.components.effects
import qs.services
import qs.utils

StyledRect {
    id: root

    required property real rootHeight

    implicitHeight: layout.implicitHeight + layout.anchors.margins * 2
    radius: Tokens.rounding.medium
    color: Colours.tPalette.m3surfaceContainer

    ColumnLayout {
        id: layout

        anchors.fill: parent
        anchors.margins: Tokens.padding.extraLarge
        spacing: Tokens.spacing.small

        RowLayout {
            Layout.fillWidth: true
            spacing: Tokens.spacing.medium

            StyledRect {
                implicitWidth: prompt.implicitWidth + Tokens.padding.medium * 2
                implicitHeight: prompt.implicitHeight + Tokens.padding.small * 2
                color: Colours.palette.m3primary
                radius: Tokens.rounding.medium

                StyledText {
                    id: prompt
                    anchors.centerIn: parent
                    text: "T"
                    color: Colours.palette.m3onPrimary
                    font: Tokens.font.mono.medium
                }
            }

            StyledText {
                Layout.fillWidth: true
                text: "Arch Teto"
                font: Tokens.font.mono.medium
                elide: Text.ElideRight
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Tokens.spacing.extraLarge

            Item {
                id: diag
                Layout.alignment: Qt.AlignVCenter
                Layout.preferredWidth: root.rootHeight * 0.15
                Layout.preferredHeight: root.rootHeight * 0.15

                Item {
                    id: archPart
                    anchors.fill: parent
                    layer.enabled: true
                    layer.effect: MultiEffect {
                        maskEnabled: true
                        maskSource: maskArch
                    }

                    Rectangle { anchors.fill: parent; color: Colours.palette.m3surfaceContainerHigh }

                    ColouredIcon {
                        anchors.fill: parent
                        source: SysInfo.osLogo
                        implicitSize: diag.height
                        colour: Colours.palette.m3primary
                    }
                }

                Item {
                    id: tetoPart
                    anchors.fill: parent
                    layer.enabled: true
                    layer.effect: MultiEffect {
                        maskEnabled: true
                        maskSource: maskTeto
                    }

                    Image {
                        anchors.fill: parent
                        source: "file://" + Paths.home + "/.face"
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                    }
                }

                Item {
                    id: maskTeto
                    anchors.fill: parent
                    visible: false
                    layer.enabled: true
                    Shape {
                        anchors.fill: parent
                        ShapePath {
                            fillColor: "white"; strokeColor: "transparent"
                            startX: 0; startY: 0
                            PathLine { x: diag.width; y: diag.height }
                            PathLine { x: 0; y: diag.height }
                            PathLine { x: 0; y: 0 }
                        }
                    }
                }

                Item {
                    id: maskArch
                    anchors.fill: parent
                    visible: false
                    layer.enabled: true
                    Shape {
                        anchors.fill: parent
                        ShapePath {
                            fillColor: "white"; strokeColor: "transparent"
                            startX: 0; startY: 0
                            PathLine { x: diag.width; y: 0 }
                            PathLine { x: diag.width; y: diag.height }
                            PathLine { x: 0; y: 0 }
                        }
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: Tokens.spacing.medium

                StyledText {
                    Layout.fillWidth: true
                    text: "USER: " + SysInfo.user
                    font: Tokens.font.mono.medium
                    elide: Text.ElideRight
                }

                StyledText {
                    Layout.fillWidth: true
                    text: "WM  : " + SysInfo.wm
                    font: Tokens.font.mono.medium
                    elide: Text.ElideRight
                }

                StyledText {
                    Layout.fillWidth: true
                    text: "UP  : " + SysInfo.uptime
                    font: Tokens.font.mono.medium
                    elide: Text.ElideRight
                }

                StyledText {
                    Layout.fillWidth: true
                    text: "BATT: " + Math.round(UPower.displayDevice.percentage * 100) + "%"
                    font: Tokens.font.mono.medium
                    elide: Text.ElideRight
                }
            }
        }
    }
}
