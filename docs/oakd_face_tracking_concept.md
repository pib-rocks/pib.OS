# Use Case: Oak-D Lite Face Tracking (Head Control)

Dieses Dokument beschreibt die Architektur und notwendigen pib.OS-Erweiterungen, um mithilfe der Oak-D Lite (RGB-D Edge AI Camera) die Kopfmotoren des pib so anzusteuern, dass der Roboter einem Gesicht folgt.

## 1. Systemarchitektur & Datenfluss

Da die Oak-D Lite die Inferenz (neuronales Netz zur Gesichts- oder Personenerkennung) direkt auf dem Device-Chip berechnet, ist die Host-CPU entlastet. Der Datenfluss gestaltet sich wie folgt:

1. **DepthAI Pipeline (Externer Prozess):** Ein dedizierter Treiber-Prozess kommuniziert via USB mit der Oak-D. Er liest die erkannten Bounding Boxes (z.B. Zentrum des Gesichts als normalisierte Koordinaten `x: 0.0 - 1.0` und `y: 0.0 - 1.0`) aus und publiziert diese als JSON via **Zenoh** auf ein Topic (z. B. `/pib/vision/faces`).
2. **pib.OS Ingestion:** Der `NetworkSubscriberBridge` Daemon in pib.OS lauscht auf `/pib/vision/faces` und aktualisiert das Zero-Copy `Blackboard` mit den Werten `face_x`, `face_y` und `face_timestamp`.
3. **Behavior Tree Execution:** Die `TickEngine` evaluiert den Baum. Logik-Knoten lesen `face_x`/`face_y`, berechnen die Motor-Vektoren und schreiben diese zurück aufs Blackboard.
4. **Motor Actuation:** `NetworkPublisherNode`s lesen die berechneten Motor-Vektoren vom Blackboard und publizieren sie via Zenoh an die Hardware-Schnittstellen der Servos (`/pib/actuators/head/pan/cmd`).

## 2. Benötigte pib.OS Erweiterungen (Neue Nodes)

Um dieses Szenario als No-Code-Block in der UI zusammenbauen zu können, müssen wir die Node-Bibliothek in `src/` erweitern:

### A. `IsDataFreshCondition` (Condition)
*   **Zweck:** Prüft, ob Sensor-Daten aktuell sind. Verhindert, dass der Kopf sich weiterbewegt, wenn das Gesicht seit 2 Sekunden aus dem Bild verschwunden ist.
*   **Config-Schema:** `{"key": "string", "max_age_ms": "integer"}`
*   **Logik:** Liest `timestamp` aus dem Blackboard. Wenn `now() - timestamp < max_age_ms` -> `Success`, sonst `Failure`.

### B. `PIDControllerNode` (Action)
*   **Zweck:** Ein proportional-integral-derivative Regler. Verwandelt die Abweichung (Error) der Gesichtsposition von der Bildmitte in eine sanfte Motor-Geschwindigkeit.
*   **Config-Schema:** 
    *   `input_key`: "face_x"
    *   `output_key`: "pan_velocity"
    *   `target`: 0.5 (Bildmitte)
    *   `p, i, d`: (Tuning-Parameter)
*   **Logik:** `error = target - read(input_key)`. Berechnet PID-Formel, schreibt Resultat in `output_key`. Gibt immer `Success` zurück.

### C. `SetBlackboardNode` (Action)
*   **Zweck:** Manuelles Setzen von Werten auf dem Blackboard (z.B. für Reset-Sequenzen).
*   **Config-Schema:** `{"key": "string", "value": "any"}`

## 3. Der Behavior Tree (pib.Cerebra Struktur)

In der Benutzeroberfläche würde der fertige Tracking-Tree so aussehen:

```text
Fallback
 ├── Sequence (Tracking)
 │    ├── IsDataFreshCondition (key: "face_x", max_age_ms: 500)
 │    ├── Parallel (PID Calculations)
 │    │    ├── PIDControllerNode (input: "face_x", output: "pan_vel", target: 0.5)
 │    │    └── PIDControllerNode (input: "face_y", output: "tilt_vel", target: 0.5)
 │    └── Parallel (Motor Commands)
 │         ├── NetworkPublisherNode (topic: "/pib/motors/pan", key: "pan_vel")
 │         └── NetworkPublisherNode (topic: "/pib/motors/tilt", key: "tilt_vel")
 └── Sequence (Reset / Idle)
      ├── SetBlackboardNode (key: "pan_vel", value: 0)
      ├── SetBlackboardNode (key: "tilt_vel", value: 0)
      ├── NetworkPublisherNode (topic: "/pib/motors/pan", key: "pan_vel")
      └── NetworkPublisherNode (topic: "/pib/motors/tilt", key: "tilt_vel")
```

## 4. Umsetzungsschritte (Epics)

1.  **Epic: Advanced Logic Nodes**
    *   Implementiere `IsDataFreshCondition`, `PIDControllerNode` und `SetBlackboardNode`.
    *   Schreibe Criterion-Benchmarks für den `PIDControllerNode` (hohe Frequenz).
2.  **Epic: DepthAI to Zenoh Bridge (Python/Rust)**
    *   Erstelle ein Standalone-Skript im Repository (Ordner `/tools/oakd_bridge`), das die DepthAI-Pipeline startet und Bounding Boxes als Zenoh-Payloads publiziert.
