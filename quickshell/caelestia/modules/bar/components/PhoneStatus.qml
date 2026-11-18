import QtQuick 2.0
import Caelestia 1.0

// Widget de estado del celular via KDE Connect
// Muestra batería, conexión y notificaciones

Rectangle {
    id: phoneStatus
    width: 80
    height: 40
    property int batteryLevel: 0
    property bool isCharging: false
    property string connectionStatus: "Desconectado"
    
    // Timer para verificar estado del celular
    Timer {
        interval: 5000  // Actualizar cada 5 segundos
        running: true
        repeat: true
        onTriggered: {
            // Verificar si KDE Connect está disponible
            // y actualizar estado del celular
            if (GlobalConfig.kdeconnect.enabled) {
                // Intentar obtener estado
                // Nota: Esto requiere que el daemon KDE Connect esté corriendo
                batteryLevel = kdeconnect.getBatteryLevel || 0
                isCharging = kdeconnect.isCharging || false
                connectionStatus = kdeconnect.status || "Conectando"
            }
        }
    }
    
    // Fondos según estado
    property color bgColor: connectionStatus.contains("Conect") ? "#2a9d8f" : "#e9c46a"
    property alias statusText: statusLabel.text
    
    // Diseño
    ColorRect {
        anchors.fill: parent
        radius: 8
        color: bgColor
    }
    
    Text {
        id: statusLabel
        anchors.centerIn: parent
        text: connectionStatus
        color: #ffffff
        font.pixelSize: 12
        font.weight: Font.DemiBold
        horizontalAlignment: Text.AlignHCenter
    }
    
    // Battery bar abajo
    Rectangle {
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottomMargin: 4
        height: 3
        color: #a8d8ef
        visible: connectionStatus.contains("Conect")
    }
    
    Text {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 18
        text: "Batt: " + batteryLevel + "%"
        color: #ffffff
        font.pixelSize: 9
    }
}