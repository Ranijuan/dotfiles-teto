import QtQuick
import QtQuick.Layouts
import Caelestia.Config
import qs.services
import qs.utils

// Desktop weather widget - floating window on desktop
// Shows current weather conditions as an overlay on the desktop

Item {
    id: root
    property int implicitHeight: 80
    property int implicitWidth: 200

    // Weather data from Weather service
    readonly property var weatherData: Weather.forecast && Weather.forecast.length > 0 ? Weather.forecast[0] : null
    readonly property var current: Weather.cc ? {
        temp: Weather.formatTemp(Weather.cc.temperature, true),
        description: Weather.cc.description || "N/A",
        weatherCode: Weather.cc.weatherCode,
        isDay: Weather.cc.isDay === 1,
        icon: Weather.cc.icon || "weather-cloudy",
        city: Weather.city || "Cali"
    } : {
        temp: "--°",
        description: "Sin datos",
        weatherCode: "0",
        isDay: true,
        icon: "weather-cloudy",
        city: "Cali"
    }

    // Background transparency
    readonly property color bgColor: current.isDay ? 
        Qt.rgba(0.1, 0.2, 0.3, 0.7) : 
        Qt.rgba(0.1, 0.1, 0.15, 0.8)

    // Weather condition icon mapping
    readonly property string conditionIcon: {
        const code = String(current.weatherCode);
        if ([0, 1, 2, 3].includes(code)) return "weather-sunny";
        if ([45, 48].includes(code)) return "weather-fog";
        if ([51, 53, 55, 56, 57].includes(code)) return "weather-rainy";
        if ([61, 63, 65, 66, 67].includes(code)) return "weather-rainy";
        if ([80, 81, 82].includes(code)) return "weather-rainy";
        if ([95, 96, 97, 99, 100].includes(code)) return "weather-lightning";
        if ([85, 86].includes(code)) return "weather-snowy";
        return "weather-cloudy";
    }

    // Position - default: top-right corner with some padding
    readonly property int posX: Config.weatherDesktop.posX || 20
    readonly property int posY: Config.weatherDesktop.posY || 20

    // Size config
    readonly property int fontSize: Config.weatherDesktop.fontSize || 14

    Component.onCompleted: Weather.reload()

    // Make this window behave as a desktop widget
    // Frameless, transparent background, stays on top or behind depending on config
    Window {
        id: weatherWindow
        visible: true
        width: implicitWidth
        height: implicitHeight
        x: posX
        y: posY
        
        // Frameless window - no title bar, borders
        WindowHintQt::Qt::FramelessWindowHint
        WindowHintQt::Qt::Tool
        WindowHintQt::Qt::WindowStaysOnBottomHint  // Stay behind windows but visible
        WindowHintQt::Qt::TransparentForMouseEvents  // Click-through if desired
        
        // Background color with alpha for transparency
        Color {
            anchors.fill: parent
            color: bgColor
            visible: Config.weatherDesktop.backgroundVisible
        }
        
        // Don't steal focus
        focus: false
        
        // Text and display
        ColumnLayout {
            anchors.centerIn: parent
            spacing: Tokens.padding.small
            
            // City name
            StyledText {
                text: current.city
                font: Tokens.font.body.builders.small.size(fontSize).weight(Font.Medium).build()
                color: Colours.palette.m3primary
                horizontalAlignment: Text.AlignHCenter
            }
            
            // Temperature
            StyledText {
                text: current.temp
                font: Tokens.font.body.builders.large.size(fontSize * 1.5).weight(Font.DemiBold).build()
                color: Colours.palette.m3primary
                horizontalAlignment: Text.AlignHCenter
            }
            
            // Condition icon + description
            RowLayout {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Tokens.padding.extraSmall
                
                MaterialIcon {
                    text: root.conditionIcon
                    fontStyle: Tokens.font.icon.builders.extraLarge.scale(1.2).build()
                    color: Colours.palette.m3secondary
                }
                
                StyledText {
                    text: current.description
                    font: Tokens.font.body.builders.small.size(fontSize).build()
                    color: Colours.palette.m3onSurfaceVariant
                    horizontalAlignment: Text.AlignHCenter
                }
            }
        }
        
        // Mouse area for dragging (if not transparentForMouseEvents)
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton
            onPressed: {
                weatherWindow.dragStartX = weatherWindow.x
                weatherWindow.dragStartY = weatherWindow.y
            }
            onReleased: {
                // Save position when released
                Config.weatherDesktop.posX = weatherWindow.x
                Config.weatherDesktop.posY = weatherWindow.y
            }
            onDoublePressed: {
                // Toggle visibility or open full dashboard
                console.log("Double-click - toggle weather widget")
            }
        }
    }
}