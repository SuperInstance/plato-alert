# plato-alert

Alert management for PLATO — creation, routing, acknowledgment, escalation.

## Overview

- **Alert** — severity-tagged alert with lifecycle (open → acknowledged → resolved)
- **AlertRule** — condition-based automatic alert creation
- **AlertRouter** — routes alerts to handlers based on severity/type
- **EscalationPolicy** — auto-escalation rules with configurable delays
- **AlertManager** — central coordinator for the full alert lifecycle

## Usage

```rust
use plato_alert::*;

let mut manager = AlertManager::new();
let alert = Alert::new(AlertSeverity::Warning, "Temperature rising")
    .with_source("room-42/sensor-temp-1");
manager.create(alert);
```

## License

Apache-2.0
