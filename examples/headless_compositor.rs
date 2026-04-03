//! Headless compositor demo — creates windows, renders a frame, and outputs stats.
//!
//! Run with: `cargo run --example headless_compositor --all-features`

use aethersafha::{Compositor, WindowState};

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .compact()
        .init();

    // Create a 1080p compositor
    let compositor = Compositor::with_resolution(1920, 1080);

    // Spawn some windows
    let terminal = compositor
        .create_window("AGNOS Terminal".into(), "terminal".into(), false)
        .expect("create terminal");
    let browser = compositor
        .create_window("Web Browser".into(), "browser".into(), false)
        .expect("create browser");
    let agent = compositor
        .create_window("Agent Monitor".into(), "agent.monitor".into(), true)
        .expect("create agent window");

    println!("Created {} windows", compositor.get_windows().len());

    // Maximize the browser
    let _ = compositor.set_window_state(browser, WindowState::Maximized);

    // Render a frame
    let pixels = compositor.render_to_vec();
    println!("Rendered frame: {} bytes", pixels.len());

    // List active windows
    for window in compositor.get_active_windows() {
        println!(
            "  [{}] {} — {:?} at ({},{}) {}x{}{}",
            if window.is_agent_window {
                "agent"
            } else {
                "app"
            },
            window.title,
            window.state,
            window.geometry.x,
            window.geometry.y,
            window.geometry.width,
            window.geometry.height,
            if Some(window.id) == compositor.focused_window() {
                " (focused)"
            } else {
                ""
            },
        );
    }

    // Clean up
    compositor.close_window(terminal).unwrap();
    compositor.close_window(browser).unwrap();
    compositor.close_window(agent).unwrap();
    println!("All windows closed.");
}
