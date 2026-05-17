# Konzept: Advanced Features (Phase 3)

Dieses Dokument beschreibt die technische Umsetzung der Advanced Features für `pib.OS` und die `pib.Cerebra` GUI. Ziel ist es, die Skalierbarkeit für komplexe Roboter-Verhalten zu gewährleisten und die Konfigurierbarkeit für Endnutzer (No-Code) zu maximieren.

## 1. Subtrees (Modular & Reusable Behaviors)

Um extrem große und komplexe Behavior Trees wartbar zu halten, müssen Bäume modularisiert werden. Ein einmal gebauter Baum (z. B. "Gesicht erkennen & fokussieren") soll als einzelner Knoten in einem anderen Baum (z. B. "Begrüßungs-Sequenz") verwendet werden können.

### Backend-Architektur (`pib.OS`)
*   **`SubtreeNode`:** Eine neue Struktur, die das `AsyncActionNode` Trait implementiert.
*   **Ausführung:** Anstatt eigenen Code auszuführen, lädt der `SubtreeNode` zur Initialisierung einen anderen Baum (referenziert über die Projekt-ID aus der SQLite-Datenbank).
*   **Memory / Blackboard Isolation:** Der Subtree erhält sein eigenes `ScopedBlackboard`. Ports können über das `port_mapping` des Subtree-Nodes an den Parent-Baum durchgereicht werden (z. B. Output-Port `target_coordinates`).

### Frontend-Architektur (`pib.Cerebra`)
*   **Workspace Integration:** Die UI erhält ein Panel mit "Gespeicherten Projekten" (aus der `GET /api/projects` API).
*   **Drag & Drop:** Nutzer können ein gespeichertes Projekt auf das Canvas ziehen. Es wird als spezieller Node gerendert (visuell unterscheidbar, z. B. mit einem "Ordner"-Icon).
*   **Drill-Down:** Ein Doppelklick auf den Subtree-Node öffnet diesen Baum in einem neuen Tab oder Modal, um ihn isoliert zu bearbeiten.

---

## 2. Dynamic Properties Panel (Node Configuration)

Aktuell sind Nodes im Frontend "dumme" Blöcke. Ein `TimeoutNode` braucht einen definierten Zeitwert, ein `ZenohPublisher` ein Topic. Diese müssen ohne Code-Änderungen über die UI konfigurierbar sein.

### Backend-Architektur (`pib.OS`)
*   **Erweiterung der Registry:** Die Route `GET /api/registry` wird erweitert. Jeder registrierte Node liefert nun zusätzlich ein `config_schema` (basierend auf JSON-Schema) zurück.
    *   *Beispiel `TimeoutNode`:* `{"properties": {"timeout_ms": {"type": "integer", "default": 1000}}}`
*   **Parser-Update:** Der JSON Tree Parser (`src/parser.rs`) wird so angepasst, dass er das `config` Objekt ausliest und bei der Instanziierung der Rust-Structs anwendet.

### Frontend-Architektur (`pib.Cerebra`)
*   **Properties Sidebar:** Wird ein Node auf dem Canvas angeklickt (Selected State), öffnet sich rechts eine Sidebar.
*   **Schema-to-Form:** Eine dynamische Formular-Komponente (z. B. via `react-jsonschema-form` oder einer leichtgewichtigen Eigenentwicklung) liest das `config_schema` des Nodes und rendert automatisch die passenden Eingabefelder (Text, Zahlen, Checkboxen, Dropdowns).
*   **State Management:** Die eingegebenen Werte werden im `data.config` Objekt des React Flow Nodes gespeichert und beim `JSON Export` automatisch an das Backend geschickt.

---

## Vorgeschlagene Epics & Stories für die Umsetzung

**Epic: Advanced Behavior Tree Features**

*   **Story 1: Backend - Dynamic Configuration & Registry Expansion (PR-1260)**
    *   Erweitern des Node-Traits und der `/api/registry` um JSON Schemas.
    *   Update des Parsers, um Laufzeit-Konfigurationen zu deserialisieren.
*   **Story 2: Frontend - Dynamic Properties Panel (PR-1261)**
    *   Implementierung der Sidebar in React, die das Schema des ausgewählten Nodes als Formular rendert und die Daten an den Node bindet.
*   **Story 3: Backend - SubtreeNode Implementation (PR-1262)**
    *   Rust-Implementierung des `SubtreeNode`, der rekursiv `TreeDef` Strukturen lädt und isolierte Blackboards erzeugt.
*   **Story 4: Frontend - Subtree D&D and Drill-Down (PR-1263)**
    *   Erweitern der UI, sodass gespeicherte Projekte als Nodes verwendet und per Doppelklick geöffnet werden können.