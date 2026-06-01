# plato-alert

> Alert management for PLATO — creation, routing, acknowledgment, and escalation

## What This Does

plato-alert manages the full lifecycle of alerts in PLATO: creation with severity levels, routing to appropriate channels based on severity thresholds, acknowledgment tracking, and automatic escalation (severity increases over time if unacknowledged).

## The Key Idea

Alerts are not just log messages — they have severity, ownership, and escalation paths. An Info alert goes to logs. A Critical alert pages the ops team. An Emergency alert that goes unacknowledged gets escalated. The system tracks which alerts are active, routes them to the right channels, and provides history per room.

## Install

```bash
cargo add plato-alert
```

## Quick Start

```rust
use plato_alert::*;

let alert = Alert::new("Overheat", AlertSeverity::Critical, "room-1", "thermal-sensor")
    .with_description("Temperature exceeded 90°C")
    .with_tag("auto");

// Route to appropriate channels
let routes = vec![
    AlertRoute::new("log-all", "logs", AlertChannel::Log),
    AlertRoute::new("pager-crit", "ops", AlertChannel::Pager)
        .with_min_severity(AlertSeverity::Critical),
];
let matched = route_alert(&alert, &routes);
// → matches both routes

// Escalate if unacknowledged
let escalated = escalate(&alert);
assert_eq!(escalated.severity, AlertSeverity::Emergency);
```

## API Reference

| Type | Description |
|---|---|
| `AlertSeverity` | `Info` < `Warning` < `Critical` < `Emergency`. `is_urgent()` for Critical+. |
| `AlertChannel` | `Log` / `Email` / `Webhook` / `Pager` / `Slack` |
| `Alert` | Builder: `new(title, severity, room_id, sensor_id).with_description().with_tag().with_metadata()` |
| `AlertRoute` | Routing rule with `min_severity` filter. `accepts(severity)` checks if an alert qualifies. |
| `AlertHistory` | Per-room alert tracking. `record()`, `active_alerts()`, `by_severity()`. |

### Functions

| Function | Description |
|---|---|
| `create_alert(title, severity, room_id, sensor_id)` | Convenience constructor |
| `route_alert(alert, routes)` | Returns routes matching the alert's severity |
| `acknowledge(&mut alert)` | Mark as acknowledged with timestamp |
| `escalate(alert)` | Create escalated copy with severity bumped one level |
| `active_alerts(alerts)` | Filter to unacknowledged alerts |

### Escalation Chain

Info → Warning → Critical → Emergency (stops at Emergency)

## Testing

19 tests: severity ordering, alert construction, builder pattern, routing by severity, acknowledgment, escalation chain, active alert filtering, history tracking, serialization.

## License

Apache-2.0
