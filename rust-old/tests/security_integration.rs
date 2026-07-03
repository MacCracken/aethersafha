use aethersafha::{SecurityUI, ThreatLevel};

#[test]
fn security_alert_lifecycle() {
    let ui = SecurityUI::new();

    let alert_id = uuid::Uuid::new_v4();
    ui.show_security_alert(aethersafha::SecurityAlert {
        id: alert_id,
        title: "Test Alert".into(),
        description: "Testing".into(),
        threat_level: ThreatLevel::Medium,
        source: "test".into(),
        timestamp: chrono::Utc::now(),
        requires_action: true,
        is_resolved: false,
    });

    let alerts = ui.get_active_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].title, "Test Alert");

    ui.dismiss_alert(alert_id).unwrap();
    // After dismissal the alert is resolved, so get_active_alerts excludes it
    let alerts = ui.get_active_alerts();
    assert!(alerts.is_empty());
}

#[test]
fn permission_request_grant_deny() {
    let ui = SecurityUI::new();

    let req1 = aethersafha::PermissionRequest {
        id: uuid::Uuid::new_v4(),
        agent_id: uuid::Uuid::new_v4(),
        agent_name: "agent-1".into(),
        permission: "screen_capture".into(),
        resource: "display:0".into(),
        reason: "needs to see".into(),
        timestamp: chrono::Utc::now(),
        is_granted: false,
    };
    let req2 = aethersafha::PermissionRequest {
        id: uuid::Uuid::new_v4(),
        agent_id: uuid::Uuid::new_v4(),
        agent_name: "agent-2".into(),
        permission: "file_access".into(),
        resource: "/tmp".into(),
        reason: "needs files".into(),
        timestamp: chrono::Utc::now(),
        is_granted: false,
    };

    let id1 = req1.id;
    let id2 = req2.id;
    ui.request_permission(req1);
    ui.request_permission(req2);

    ui.grant_permission(id1).unwrap();
    ui.deny_permission(id2).unwrap();

    // req1 is granted (filtered out by get_pending_permissions), req2 is removed by deny
    let requests = ui.get_pending_permissions();
    assert!(requests.is_empty());
}

#[test]
fn human_override_workflow() {
    let ui = SecurityUI::new();

    let id = ui.request_human_override(
        "test-agent".into(),
        "delete_file".into(),
        "cleanup needed".into(),
    );

    let overrides = ui.get_override_requests();
    assert_eq!(overrides.len(), 1);
    assert!(!overrides[0].is_approved);

    ui.approve_override(id, "admin".into()).unwrap();
    // After approval, get_override_requests excludes approved entries
    let overrides = ui.get_override_requests();
    assert!(overrides.is_empty());
}

#[test]
fn agent_permissions_set_and_revoke() {
    let ui = SecurityUI::new();
    let agent_id = uuid::Uuid::new_v4();

    ui.set_agent_permissions(
        agent_id,
        "test-agent".into(),
        vec!["screen_capture".into(), "clipboard".into()],
    );

    ui.revoke_agent_permissions(agent_id).unwrap();

    let err = ui.revoke_agent_permissions(agent_id);
    assert!(err.is_err());
}
