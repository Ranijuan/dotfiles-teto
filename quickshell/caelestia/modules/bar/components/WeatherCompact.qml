import QtQuick
import QtQuick.Layouts
import Caelestia.Config
import qs.services
import qs.utils

// Compact weather widget for the bar
// Shows: city, temperature, condition icon
// Clickable - opens full WeatherTab dashboard

Item {
    id: root
    property int implicitHeight: 40
    property int implicitWidth: 120

    // Weather service
    readonly property var weatherData: Weather.forecast && Weather.forecast.length > 0 ? Weather.forecast[0] : null
    readonly property var current: Weather.cc ? {
        temp: Weather.formatTemp(Weather.cc.temperature, true),
        description: Weather.cc.description || "",
        weatherCode: Weather.cc.weatherCode,
        isDay: Weather.cc.isDay === 1,
        icon: Weather.cc.icon || "weather-cloudy"
    } : {
        temp: "--°",
        description: "N/A",
        weatherCode: "0",
        isDay: true,
        icon: "weather-cloudy"
    }

    // Background color based on day/night
    readonly property color bgColor: current.isDay ? Colours.tPalette.m3surface : Colours.tPalette.m3surfaceContainer

    readonly property string conditionIcon: {
        const code = String(current.weatherCode);
        // Map weather codes to icons
        if ([80, 81, 82].includes(code)) return "weather-rainy";
        if ([95, 96, 97, 99, 100].includes(code)) return "weather-lightning";
        if ([51, 53, 55, 56, 57].includes(code)) return "weather-rainy";
        if ([61, 63, 65, 66, 67].includes(code)) return "weather-rainy";
        if ([85, 86].includes(code)) return "weather-snowy";
        if ([96].includes(code)) return "weather-hail";
        return "weather-sunny";
    }

    readonly property string tempText: current.temp !== "N/A" ? current.temp : "--°"

    Component.onCompleted: Weather.reload()

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            // Open full weather dashboard
            if (Qt.platform.isWayland) {
                // Try to open the dashboard weather tab
                // This would need a way to switch to dashboard view
                console.log("Opening full weather dashboard")
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        radius: Tokens.rounding.medium
        color: bgColor
        opacity: 0.8
        z: 1

        // Weather city
        StyledText {
            text: Weather.city || "---"
            font: Tokens.font.body.builders.medium.size(12).weight(Font.Medium).build()
            color: Colours.palette.m3primary
            anchors.left: parent.left
            anchors.leftMargin: Tokens.padding.small
            anchors.verticalCenter: parent.verticalCenter
        }

        // Temperature
        StyledText {
            text: root.tempText
            font: Tokens.font.body.builders.large.size(18).weight(Font.Medium).build()
            color: Colours.palette.m3primary
            anchors.right: parent.right
            anchors.rightMargin: Tokens.padding.small
            anchors.verticalCenter: parent.verticalCenter
        }

        // Condition icon
        MaterialIcon {
            text: root.conditionIcon
            fontStyle: Tokens.font.icon.builders.extraLarge.scale(1.5).build()
            color: Colours.palette.m3secondary
            anchors.right: parent.right
            anchors.rightMargin: Tokens.padding.extraLarge
            anchors.verticalCenter: parent.verticalCenter
        }
    }
}