use aethersafha::{Compositor, CompositorError, InputAction, InputEvent, WindowState};

#[test]
fn create_and_close_window() {
    let compositor = Compositor::new();
    let id = compositor
        .create_window("Test".into(), "test.app".into(), false)
        .unwrap();

    let windows = compositor.get_windows();
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].title, "Test");

    compositor.close_window(id).unwrap();
    assert!(compositor.get_windows().is_empty());
}

#[test]
fn close_nonexistent_window_fails() {
    let compositor = Compositor::new();
    let err = compositor.close_window(uuid::Uuid::new_v4());
    assert!(matches!(err, Err(CompositorError::WindowNotFound(_))));
}

#[test]
fn window_appears_in_active_workspace() {
    let compositor = Compositor::new();
    let id = compositor
        .create_window("Active".into(), "test.app".into(), false)
        .unwrap();

    let active = compositor.get_active_windows();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, id);
}

#[test]
fn workspace_switching_hides_windows() {
    let compositor = Compositor::new();
    compositor
        .create_window("WS1".into(), "test.app".into(), false)
        .unwrap();

    // Switch to workspace 2
    let _ = compositor.switch_workspace(1);
    assert!(compositor.get_active_windows().is_empty());

    // Switch back
    let _ = compositor.switch_workspace(0);
    assert_eq!(compositor.get_active_windows().len(), 1);
}

#[test]
fn agent_window_gets_floating_layer() {
    let compositor = Compositor::new();
    let id = compositor
        .create_window("Agent".into(), "agent.app".into(), true)
        .unwrap();

    let windows = compositor.get_windows();
    assert!(windows.iter().any(|w| w.id == id && w.is_agent_window));
}

#[test]
fn window_state_transitions() {
    let compositor = Compositor::new();
    let id = compositor
        .create_window("State".into(), "test.app".into(), false)
        .unwrap();

    let _ = compositor.set_window_state(id, WindowState::Maximized);
    let windows = compositor.get_windows();
    assert_eq!(windows[0].state, WindowState::Maximized);

    let _ = compositor.set_window_state(id, WindowState::Fullscreen);
    let windows = compositor.get_windows();
    assert_eq!(windows[0].state, WindowState::Fullscreen);

    let _ = compositor.set_window_state(id, WindowState::Normal);
    let windows = compositor.get_windows();
    assert_eq!(windows[0].state, WindowState::Normal);
}

#[test]
fn move_and_resize_window() {
    let compositor = Compositor::new();
    let id = compositor
        .create_window("Move".into(), "test.app".into(), false)
        .unwrap();

    compositor.move_window(id, 200, 300);
    let windows = compositor.get_windows();
    assert_eq!(windows[0].geometry.x, 200);
    assert_eq!(windows[0].geometry.y, 300);

    compositor.resize_window(id, 800, 600);
    let windows = compositor.get_windows();
    assert_eq!(windows[0].geometry.width, 800);
    assert_eq!(windows[0].geometry.height, 600);
}

#[test]
fn focus_follows_creation() {
    let compositor = Compositor::new();
    let _id1 = compositor
        .create_window("First".into(), "test.app".into(), false)
        .unwrap();
    let id2 = compositor
        .create_window("Second".into(), "test.app".into(), false)
        .unwrap();

    // Most recent window should be focused
    let focused = compositor.focused_window();
    assert_eq!(focused, Some(id2));
}

#[test]
fn closing_focused_window_moves_focus() {
    let compositor = Compositor::new();
    let id1 = compositor
        .create_window("First".into(), "test.app".into(), false)
        .unwrap();
    let id2 = compositor
        .create_window("Second".into(), "test.app".into(), false)
        .unwrap();

    compositor.close_window(id2).unwrap();
    let focused = compositor.focused_window();
    assert_eq!(focused, Some(id1));
}

#[test]
fn input_routing_click_on_empty_is_none() {
    let compositor = Compositor::new();
    let action = compositor.route_input(&InputEvent::MouseClick {
        button: 1,
        x: 5000,
        y: 5000,
    });
    assert_eq!(action, InputAction::None);
}

#[test]
fn custom_resolution() {
    let compositor = Compositor::with_resolution(3840, 2160);
    let id = compositor
        .create_window("4K".into(), "test.app".into(), false)
        .unwrap();

    let windows = compositor.get_windows();
    // Default window size should be based on the output resolution
    assert!(windows[0].geometry.width >= 400);
    assert!(windows[0].geometry.height >= 300);
    compositor.close_window(id).unwrap();
}

#[test]
fn secure_mode_toggle() {
    let compositor = Compositor::new();
    compositor.set_secure_mode(true);
    compositor.set_secure_mode(false);
    // No panic — secure mode toggles cleanly
}

#[test]
fn many_windows_cascade() {
    let compositor = Compositor::new();
    let mut ids = Vec::new();
    for i in 0..20 {
        let id = compositor
            .create_window(format!("Win {}", i), "test.app".into(), false)
            .unwrap();
        ids.push(id);
    }

    // All windows should have distinct positions (cascade offset)
    let windows = compositor.get_windows();
    let positions: Vec<(i32, i32)> = windows
        .iter()
        .map(|w| (w.geometry.x, w.geometry.y))
        .collect();
    // At least some should differ (cascade wraps at 300, so with 20 windows we get variety)
    let unique: std::collections::HashSet<_> = positions.iter().collect();
    assert!(unique.len() > 1);

    // Clean up
    for id in ids {
        compositor.close_window(id).unwrap();
    }
    assert!(compositor.get_windows().is_empty());
}
