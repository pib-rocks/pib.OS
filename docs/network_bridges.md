# Konzept: pib.OS Network Bridges (Phase 1)

## Vision
Damit der Behavior Tree nicht im luftleeren Raum agiert, sondern echte Hardware (Aktuatoren, Motoren) ansteuern und Sensorik (Kameras, Lidar) auslesen kann, benötigt `pib.OS` eine Abstraktionsschicht. Diese Schicht soll netzwerkagnostisch sein, sodass in Zukunft sowohl ROS2 (via DDS) als auch Eclipse Zenoh nahtlos angebunden werden können.

## Architektur-Komponenten

### 1. NetworkBackend (Trait)
Ein asynchrones Interface in Rust, das die grundlegenden Netzwerkoperationen abstrahiert.
*   `async fn publish(topic: &str, payload: &[u8]) -> Result<(), Error>`
*   `async fn subscribe(topic: &str) -> BoxStream<Payload>` (oder ähnliches Callback/Channel-System)

*Dadurch können wir für die TDD-Tests ein `MockNetworkBackend` schreiben, während wir später ein `ZenohBackend` oder `Ros2Backend` als "Drop-In" verwenden können.*

### 2. NetworkPublisherNode (Action Node)
Ein Behavior Tree Action Node, der das `NetworkBackend` nutzt.
*   **Funktion:** Liest zur Laufzeit einen definierten Wert aus dem `Blackboard` (z.B. die berechnete Soll-Geschwindigkeit), serialisiert ihn zu JSON/Bytes und schickt ihn an ein Topic.
*   **Status:** Liefert `Success` wenn das Senden geklappt hat, `Failure` wenn das Netzwerk weg ist, oder `Running` wenn ein blockierender Sende-Ablauf noch wartet.

### 3. NetworkSubscriberBridge (Daemon)
Sensordaten fließen asynchron und kontinuierlich – sie passen nicht in das synchrone "Tick"-Modell eines einzelnen Action Nodes.
*   **Funktion:** Ein Tokio-Hintergrund-Task (Daemon), der auf einem Topic lauscht (z.B. `/sensor/lidar/distance`).
*   **Aktion:** Sobald ein neues Paket ankommt, schreibt der Daemon den Wert sicher per Lock in das Zero-Copy `Blackboard` des Behavior Trees.
*   **Integration:** Ein BT `Condition Node` kann diesen Wert anschließend in Bruchteilen einer Millisekunde synchron lesen, ohne auf das Netzwerk warten zu müssen.

## Epics & Stories
*   **Epic:** Hardware & Network Interoperability
    *   **Story 1:** Implement NetworkBackend Trait & Mock (PR-1241)
    *   **Story 2:** Implement NetworkPublisherNode for BT Execution (PR-1242)
    *   **Story 3:** Implement NetworkSubscriberBridge Daemon for Blackboard Injection (PR-1243)
