//! # plato-alert
//!
//! Alert management for PLATO — creation, routing, acknowledgment, escalation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Alert severity ───────────────────────────────────────────────────

/// Severity level for an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl AlertSeverity {
    /// Numeric level (higher = more severe).
    pub fn level(&self) -> u8 {
        match self {
            AlertSeverity::Info => 0,
            AlertSeverity::Warning => 1,
            AlertSeverity::Critical => 2,
            AlertSeverity::Emergency => 3,
        }
    }

    /// True if this severity requires immediate attention.
    pub fn is_urgent(&self) -> bool {
        matches!(self, AlertSeverity::Critical | AlertSeverity::Emergency)
    }
}

// ── Alert route ──────────────────────────────────────────────────────

/// Routing destination for an alert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertRoute {
    pub id: String,
    pub target: String,
    pub channel: AlertChannel,
    pub min_severity: AlertSeverity,
}

impl AlertRoute {
    /// Create a new alert route.
    pub fn new(id: &str, target: &str, channel: AlertChannel) -> Self {
        Self {
            id: id.to_string(),
            target: target.to_string(),
            channel,
            min_severity: AlertSeverity::Info,
        }
    }

    /// Builder: set minimum severity for this route.
    pub fn with_min_severity(mut self, severity: AlertSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Check if an alert's severity meets this route's minimum.
    pub fn accepts(&self, severity: AlertSeverity) -> bool {
        severity.level() >= self.min_severity.level()
    }
}

/// Channel types for alert delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertChannel {
    Log,
    Email,
    Webhook,
    Pager,
    Slack,
}

// ── Alert ────────────────────────────────────────────────────────────

/// A single alert in the PLATO system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub room_id: String,
    pub sensor_id: String,
    pub acknowledged: bool,
    pub escalated: bool,
    pub created_at: u64,
    pub acknowledged_at: Option<u64>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert.
    pub fn new(title: &str, severity: AlertSeverity, room_id: &str, sensor_id: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: String::new(),
            severity,
            room_id: room_id.to_string(),
            sensor_id: sensor_id.to_string(),
            acknowledged: false,
            escalated: false,
            created_at: now_millis(),
            acknowledged_at: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Builder: set description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Builder: add a tag.
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Builder: add metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Age in seconds.
    pub fn age_seconds(&self, now: u64) -> u64 {
        now.saturating_sub(self.created_at) / 1000
    }
}

// ── Alert history ────────────────────────────────────────────────────

/// History tracker for alerts in a room or fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistory {
    pub room_id: String,
    pub alerts: Vec<Alert>,
}

impl AlertHistory {
    /// Create a new alert history for a room.
    pub fn new(room_id: &str) -> Self {
        Self {
            room_id: room_id.to_string(),
            alerts: Vec::new(),
        }
    }

    /// Record an alert.
    pub fn record(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }

    /// Get all active (unacknowledged) alerts.
    pub fn active_alerts(&self) -> Vec<&Alert> {
        self.alerts.iter().filter(|a| !a.acknowledged).collect()
    }

    /// Get all alerts at or above a severity.
    pub fn by_severity(&self, min_severity: AlertSeverity) -> Vec<&Alert> {
        self.alerts
            .iter()
            .filter(|a| a.severity.level() >= min_severity.level())
            .collect()
    }

    /// Total alert count.
    pub fn count(&self) -> usize {
        self.alerts.len()
    }
}

// ── Core functions ───────────────────────────────────────────────────

/// Create a new alert (convenience wrapper).
pub fn create_alert(
    title: &str,
    severity: AlertSeverity,
    room_id: &str,
    sensor_id: &str,
) -> Alert {
    Alert::new(title, severity, room_id, sensor_id)
}

/// Route an alert to matching routes based on severity.
pub fn route_alert<'a>(alert: &Alert, routes: &'a [AlertRoute]) -> Vec<&'a AlertRoute> {
    routes
        .iter()
        .filter(|r| r.accepts(alert.severity))
        .collect()
}

/// Acknowledge an alert.
pub fn acknowledge(alert: &mut Alert) {
    alert.acknowledged = true;
    alert.acknowledged_at = Some(now_millis());
}

/// Escalate an alert (increases severity if not already Emergency, returns a new escalated copy).
pub fn escalate(alert: &Alert) -> Alert {
    let new_severity = match alert.severity {
        AlertSeverity::Info => AlertSeverity::Warning,
        AlertSeverity::Warning => AlertSeverity::Critical,
        AlertSeverity::Critical => AlertSeverity::Emergency,
        AlertSeverity::Emergency => AlertSeverity::Emergency,
    };
    let mut escalated = alert.clone();
    escalated.id = Uuid::new_v4();
    escalated.severity = new_severity;
    escalated.escalated = true;
    escalated.created_at = now_millis();
    escalated
}

/// Get active (unacknowledged) alerts from a list.
pub fn active_alerts(alerts: &[Alert]) -> Vec<&Alert> {
    alerts.iter().filter(|a| !a.acknowledged).collect()
}

// ── Helpers ──────────────────────────────────────────────────────────

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(AlertSeverity::Info < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::Critical);
        assert!(AlertSeverity::Critical < AlertSeverity::Emergency);
    }

    #[test]
    fn severity_level() {
        assert_eq!(AlertSeverity::Info.level(), 0);
        assert_eq!(AlertSeverity::Emergency.level(), 3);
    }

    #[test]
    fn severity_is_urgent() {
        assert!(!AlertSeverity::Info.is_urgent());
        assert!(!AlertSeverity::Warning.is_urgent());
        assert!(AlertSeverity::Critical.is_urgent());
        assert!(AlertSeverity::Emergency.is_urgent());
    }

    #[test]
    fn alert_new() {
        let a = Alert::new("Overheat", AlertSeverity::Critical, "room-1", "s-thermal-1");
        assert_eq!(a.title, "Overheat");
        assert_eq!(a.severity, AlertSeverity::Critical);
        assert_eq!(a.room_id, "room-1");
        assert!(!a.acknowledged);
        assert!(!a.id.is_nil());
    }

    #[test]
    fn alert_builder() {
        let a = Alert::new("Test", AlertSeverity::Warning, "r1", "s1")
            .with_description("Something happened")
            .with_tag("auto")
            .with_metadata("source", "plato-tiles");

        assert_eq!(a.description, "Something happened");
        assert!(a.tags.contains(&"auto".to_string()));
        assert_eq!(a.metadata.get("source").unwrap(), "plato-tiles");
    }

    #[test]
    fn alert_age() {
        let a = Alert::new("Test", AlertSeverity::Info, "r1", "s1");
        let now = a.created_at + 5000;
        assert_eq!(a.age_seconds(now), 5);
    }

    #[test]
    fn create_alert_function() {
        let a = create_alert("Disk Full", AlertSeverity::Emergency, "r2", "s-disk");
        assert_eq!(a.title, "Disk Full");
    }

    #[test]
    fn alert_route_new() {
        let route = AlertRoute::new("pager-ops", "ops-team", AlertChannel::Pager);
        assert_eq!(route.id, "pager-ops");
        assert_eq!(route.channel, AlertChannel::Pager);
    }

    #[test]
    fn alert_route_accepts_severity() {
        let route = AlertRoute::new("r1", "team", AlertChannel::Slack)
            .with_min_severity(AlertSeverity::Critical);

        assert!(!route.accepts(AlertSeverity::Info));
        assert!(!route.accepts(AlertSeverity::Warning));
        assert!(route.accepts(AlertSeverity::Critical));
        assert!(route.accepts(AlertSeverity::Emergency));
    }

    #[test]
    fn route_alert_filters() {
        let routes = vec![
            AlertRoute::new("log-all", "logs", AlertChannel::Log),
            AlertRoute::new("pager-crit", "ops", AlertChannel::Pager)
                .with_min_severity(AlertSeverity::Critical),
        ];
        let alert = Alert::new("Warn", AlertSeverity::Warning, "r1", "s1");
        let matched = route_alert(&alert, &routes);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "log-all");
    }

    #[test]
    fn acknowledge_alert() {
        let mut a = Alert::new("Test", AlertSeverity::Warning, "r1", "s1");
        assert!(!a.acknowledged);
        acknowledge(&mut a);
        assert!(a.acknowledged);
        assert!(a.acknowledged_at.is_some());
    }

    #[test]
    fn escalate_alert() {
        let a = Alert::new("Test", AlertSeverity::Warning, "r1", "s1");
        let esc = escalate(&a);
        assert_eq!(esc.severity, AlertSeverity::Critical);
        assert!(esc.escalated);
        assert_ne!(esc.id, a.id);
    }

    #[test]
    fn escalate_emergency_stays() {
        let a = Alert::new("Max", AlertSeverity::Emergency, "r1", "s1");
        let esc = escalate(&a);
        assert_eq!(esc.severity, AlertSeverity::Emergency);
    }

    #[test]
    fn escalate_chain_info_to_emergency() {
        let a = Alert::new("Low", AlertSeverity::Info, "r1", "s1");
        let e1 = escalate(&a);
        assert_eq!(e1.severity, AlertSeverity::Warning);
        let e2 = escalate(&e1);
        assert_eq!(e2.severity, AlertSeverity::Critical);
        let e3 = escalate(&e2);
        assert_eq!(e3.severity, AlertSeverity::Emergency);
    }

    #[test]
    fn active_alerts_filters() {
        let mut a1 = Alert::new("A1", AlertSeverity::Info, "r1", "s1");
        let a2 = Alert::new("A2", AlertSeverity::Warning, "r1", "s1");
        acknowledge(&mut a1);
        let alerts = [a1, a2];
        let active = active_alerts(&alerts);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "A2");
    }

    #[test]
    fn alert_history_record_and_active() {
        let mut history = AlertHistory::new("room-1");
        let mut a1 = Alert::new("A1", AlertSeverity::Warning, "room-1", "s1");
        let a2 = Alert::new("A2", AlertSeverity::Critical, "room-1", "s2");
        acknowledge(&mut a1);
        history.record(a1);
        history.record(a2);
        assert_eq!(history.count(), 2);
        assert_eq!(history.active_alerts().len(), 1);
    }

    #[test]
    fn alert_history_by_severity() {
        let mut history = AlertHistory::new("room-1");
        history.record(Alert::new("Info", AlertSeverity::Info, "room-1", "s1"));
        history.record(Alert::new("Warn", AlertSeverity::Warning, "room-1", "s1"));
        history.record(Alert::new("Crit", AlertSeverity::Critical, "room-1", "s1"));
        let criticals = history.by_severity(AlertSeverity::Critical);
        assert_eq!(criticals.len(), 1);
        assert_eq!(criticals[0].title, "Crit");
    }

    #[test]
    fn alert_serializes() {
        let a = Alert::new("Test", AlertSeverity::Critical, "r1", "s1");
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("Critical"));
        assert!(json.contains("r1"));
    }

    #[test]
    fn alert_deserializes() {
        let a = Alert::new("Test", AlertSeverity::Warning, "r1", "s1")
            .with_tag("test-tag");
        let json = serde_json::to_string(&a).unwrap();
        let back: Alert = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Test");
        assert!(back.tags.contains(&"test-tag".to_string()));
    }
}
