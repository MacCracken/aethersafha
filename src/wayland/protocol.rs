#![allow(dead_code)]
//! Protocol bridge and extension types for Wayland integration.

use std::collections::HashMap;

use crate::compositor::{Compositor, InputAction, InputEvent, SurfaceId, WindowState};

use super::types::*;

/// Actions the compositor should take in response to protocol events.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ProtocolAction {
    /// Create a new window for a client surface.
    CreateWindow {
        client_id: u32,
        surface_id: SurfaceId,
        title: String,
        app_id: String,
    },
    /// Destroy a surface and its window.
    DestroyWindow { surface_id: SurfaceId },
    /// Submit a pixel buffer to a window.
    SubmitBuffer {
        surface_id: SurfaceId,
        buffer: ShmBufferInfo,
    },
    /// Set window title.
    SetTitle {
        surface_id: SurfaceId,
        title: String,
    },
    /// Set window app_id.
    SetAppId {
        surface_id: SurfaceId,
        app_id: String,
    },
    /// Client requests window move (interactive drag).
    RequestMove { surface_id: SurfaceId },
    /// Client requests window resize from an edge.
    RequestResize { surface_id: SurfaceId, edge: u32 },
    /// Client requests maximized state.
    SetMaximized {
        surface_id: SurfaceId,
        maximized: bool,
    },
    /// Client requests fullscreen state.
    SetFullscreen {
        surface_id: SurfaceId,
        fullscreen: bool,
    },
    /// Client requests minimize.
    SetMinimized { surface_id: SurfaceId },
    /// Client sets min/max size constraints.
    SetSizeBounds {
        surface_id: SurfaceId,
        min_size: Option<(u32, u32)>,
        max_size: Option<(u32, u32)>,
    },
    /// Client acknowledged a configure event.
    AckConfigure { surface_id: SurfaceId, serial: u32 },
    /// Client committed a surface (buffer attached + damage).
    SurfaceCommit { surface_id: SurfaceId },
    /// Send configure event to client.
    SendConfigure { configure: ToplevelConfigure },
    /// Forward pointer event to focused client.
    ForwardPointer { event: WaylandPointerEvent },
    /// Forward keyboard event to focused client.
    ForwardKeyboard { event: WaylandKeyboardEvent },
    /// A new client connected.
    ClientConnected { client_id: u32 },
    /// A client disconnected — clean up all its surfaces.
    ClientDisconnected { client_id: u32 },
    /// Set clipboard selection for a surface.
    SetSelection {
        surface_id: SurfaceId,
        mime_types: Vec<String>,
    },
    /// Start a drag-and-drop operation.
    StartDrag {
        source: SurfaceId,
        icon: Option<SurfaceId>,
        mime_types: Vec<String>,
    },
    /// Enable text input on a surface.
    TextInputEnable { surface_id: SurfaceId },
    /// Disable text input on a surface.
    TextInputDisable { surface_id: SurfaceId },
    /// Commit text input.
    TextInputCommit { surface_id: SurfaceId, text: String },
    /// Set decoration mode for a surface.
    SetDecorationMode {
        surface_id: SurfaceId,
        mode: DecorationMode,
    },
    /// Set viewport for a surface.
    SetViewport {
        surface_id: SurfaceId,
        source: Option<ViewportSource>,
        destination: Option<(u32, u32)>,
    },
    /// Set fractional scale for a surface.
    SetFractionalScale {
        surface_id: SurfaceId,
        scale_120: u32,
    },
}

// ============================================================================
// Protocol extension types (data device, text input, decorations, viewporter,
// fractional scale)
// ============================================================================

/// Data device manager — manages clipboard and drag-and-drop.
#[derive(Debug, Clone)]
pub struct DataDeviceManager {
    pub selections: HashMap<SurfaceId, DataOffer>,
    pub drag_source: Option<DragState>,
}

#[derive(Debug, Clone)]
pub struct DataOffer {
    pub mime_types: Vec<String>,
    pub source_surface: SurfaceId,
    pub serial: u32,
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub source_surface: SurfaceId,
    pub icon_surface: Option<SurfaceId>,
    pub mime_types: Vec<String>,
    pub position: (f64, f64),
    pub active: bool,
}

impl DataDeviceManager {
    pub fn new() -> Self {
        Self {
            selections: HashMap::new(),
            drag_source: None,
        }
    }

    pub fn set_selection(&mut self, surface_id: SurfaceId, mime_types: Vec<String>, serial: u32) {
        self.selections.insert(
            surface_id,
            DataOffer {
                mime_types,
                source_surface: surface_id,
                serial,
            },
        );
    }

    pub fn clear_selection(&mut self, surface_id: &SurfaceId) {
        self.selections.remove(surface_id);
    }

    pub fn start_drag(
        &mut self,
        source_surface: SurfaceId,
        icon_surface: Option<SurfaceId>,
        mime_types: Vec<String>,
    ) {
        self.drag_source = Some(DragState {
            source_surface,
            icon_surface,
            mime_types,
            position: (0.0, 0.0),
            active: true,
        });
    }

    pub fn end_drag(&mut self) {
        self.drag_source = None;
    }

    pub fn get_selection(&self, surface_id: &SurfaceId) -> Option<&DataOffer> {
        self.selections.get(surface_id)
    }
}

impl Default for DataDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Text input state for IME integration (zwp_text_input_v3).
#[derive(Debug, Clone)]
pub struct TextInputState {
    pub surface_id: Option<SurfaceId>,
    pub enabled: bool,
    pub content_type: ContentType,
    pub surrounding_text: String,
    pub cursor_position: u32,
    pub preedit: Option<PreeditState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ContentType {
    #[default]
    Normal,
    Password,
    Email,
    Number,
    Phone,
    Url,
    Terminal,
}

#[derive(Debug, Clone)]
pub struct PreeditState {
    pub text: String,
    pub cursor_begin: i32,
    pub cursor_end: i32,
}

impl TextInputState {
    pub fn new() -> Self {
        Self {
            surface_id: None,
            enabled: false,
            content_type: ContentType::default(),
            surrounding_text: String::new(),
            cursor_position: 0,
            preedit: None,
        }
    }

    pub fn enable(&mut self, surface_id: SurfaceId) {
        self.surface_id = Some(surface_id);
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.surface_id = None;
        self.preedit = None;
    }

    pub fn set_surrounding_text(&mut self, text: String, cursor_position: u32) {
        self.surrounding_text = text;
        self.cursor_position = cursor_position;
    }

    pub fn commit_preedit(&mut self) -> Option<String> {
        self.preedit.take().map(|p| p.text)
    }

    pub fn clear_preedit(&mut self) {
        self.preedit = None;
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Decoration mode negotiation (xdg_decoration_unstable_v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DecorationMode {
    ClientSide,
    #[default]
    ServerSide,
}

#[derive(Debug, Clone)]
pub struct DecorationState {
    pub surface_id: SurfaceId,
    pub preferred: DecorationMode,
    pub current: DecorationMode,
}

impl DecorationState {
    pub fn new(surface_id: SurfaceId) -> Self {
        Self {
            surface_id,
            preferred: DecorationMode::ServerSide,
            current: DecorationMode::ServerSide,
        }
    }

    /// Negotiate decoration mode. Returns the mode that will be used.
    /// The compositor prefers server-side decorations; the client's preference
    /// is honoured only when it also requests server-side, otherwise the
    /// compositor falls back to client-side if the client insists.
    pub fn negotiate(&mut self) -> DecorationMode {
        self.current = self.preferred;
        self.current
    }
}

/// Viewport state for surface scaling (wp_viewporter).
#[derive(Debug, Clone)]
pub struct ViewportState {
    pub surface_id: SurfaceId,
    pub source: Option<ViewportSource>,
    pub destination: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportSource {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ViewportState {
    pub fn new(surface_id: SurfaceId) -> Self {
        Self {
            surface_id,
            source: None,
            destination: None,
        }
    }

    pub fn set_source(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.source = Some(ViewportSource {
            x,
            y,
            width,
            height,
        });
    }

    pub fn set_destination(&mut self, width: u32, height: u32) {
        self.destination = Some((width, height));
    }

    /// Returns the effective size: destination if set, otherwise source dimensions
    /// (truncated to u32), otherwise None.
    pub fn effective_size(&self) -> Option<(u32, u32)> {
        if let Some(dest) = self.destination {
            Some(dest)
        } else {
            self.source.map(|s| (s.width as u32, s.height as u32))
        }
    }
}

/// Fractional scale factor (wp_fractional_scale_v1).
#[derive(Debug, Clone)]
pub struct FractionalScale {
    pub surface_id: SurfaceId,
    /// Scale factor in 1/120ths (e.g., 120 = 1x, 150 = 1.25x, 240 = 2x).
    pub scale_120: u32,
}

impl FractionalScale {
    pub fn new(surface_id: SurfaceId, scale_120: u32) -> Self {
        Self {
            surface_id,
            scale_120,
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_120 as f64 / 120.0
    }

    pub fn from_scale(surface_id: SurfaceId, scale: f64) -> Self {
        Self {
            surface_id,
            scale_120: (scale * 120.0).round() as u32,
        }
    }
}

/// Protocol bridge between Wayland protocol events and the AGNOS compositor.
///
/// This is feature-independent — the logic works identically in stub and live modes.
/// The live mode feeds real protocol events; the stub can be driven programmatically.
#[derive(Debug)]
pub struct ProtocolBridge {
    pub surface_map: SurfaceMap,
    pub clients: ClientRegistry,
    pub serial: SerialCounter,
    pub pointer_focus: PointerFocus,
    pub keyboard_focus: KeyboardFocus,
    pub toplevels: HashMap<SurfaceId, XdgToplevelTracker>,
    pub output: OutputInfo,
    pub seat_caps: SeatCapabilities,
    pending_actions: Vec<ProtocolAction>,
}

impl ProtocolBridge {
    pub fn new() -> Self {
        Self {
            surface_map: SurfaceMap::new(),
            clients: ClientRegistry::new(),
            serial: SerialCounter::new(),
            pointer_focus: PointerFocus::default(),
            keyboard_focus: KeyboardFocus::default(),
            toplevels: HashMap::new(),
            output: OutputInfo::default(),
            seat_caps: SeatCapabilities::default(),
            pending_actions: Vec::new(),
        }
    }

    /// Handle a new client connection.
    pub fn client_connect(&mut self, pid: Option<u32>) -> u32 {
        let id = if let Some(pid) = pid {
            self.clients.register_with_pid(pid)
        } else {
            self.clients.register()
        };
        self.pending_actions
            .push(ProtocolAction::ClientConnected { client_id: id });
        id
    }

    /// Handle client disconnection — removes all its surfaces.
    pub fn client_disconnect(&mut self, client_id: u32) -> Vec<SurfaceId> {
        let mut removed_surfaces = Vec::new();
        if let Some(info) = self.clients.unregister(client_id) {
            for surface_id in &info.surfaces {
                self.surface_map.unregister(surface_id);
                self.toplevels.remove(surface_id);
                self.pending_actions.push(ProtocolAction::DestroyWindow {
                    surface_id: *surface_id,
                });
                removed_surfaces.push(*surface_id);
            }
            // Clear focus if it belonged to this client
            if let Some(focused) = self.pointer_focus.surface_id
                && info.surfaces.contains(&focused)
            {
                self.pointer_focus.surface_id = None;
            }
            if let Some(focused) = self.keyboard_focus.surface_id
                && info.surfaces.contains(&focused)
            {
                self.keyboard_focus.surface_id = None;
            }
        }
        self.pending_actions
            .push(ProtocolAction::ClientDisconnected { client_id });
        removed_surfaces
    }

    /// Create a new wl_surface for a client.
    pub fn create_surface(&mut self, client_id: u32) -> Option<(SurfaceId, u32)> {
        let surface_id = uuid::Uuid::new_v4();
        let proto_id = self.surface_map.register(surface_id);
        if let Some(client) = self.clients.get_mut(client_id) {
            client.add_surface(surface_id);
        }
        Some((surface_id, proto_id))
    }

    /// Handle xdg_surface.get_toplevel — creates the toplevel tracker and triggers window creation.
    pub fn create_toplevel(&mut self, surface_id: SurfaceId, client_id: u32) -> &ToplevelConfigure {
        let tracker = XdgToplevelTracker::new(surface_id);
        self.toplevels.insert(surface_id, tracker);

        // Send initial configure
        let serial = self.serial.next_serial();
        let configure = ToplevelConfigure::initial(surface_id, serial);

        self.pending_actions.push(ProtocolAction::CreateWindow {
            client_id,
            surface_id,
            title: String::new(),
            app_id: String::new(),
        });

        // Safe: we just inserted the tracker above
        let tracker = self.toplevels.get_mut(&surface_id).expect("just inserted");
        tracker.send_configure(configure)
    }

    /// Handle xdg_toplevel.set_title.
    pub fn set_title(&mut self, surface_id: SurfaceId, title: String) {
        if let Some(tracker) = self.toplevels.get_mut(&surface_id) {
            tracker.title = Some(title.clone());
        }
        self.pending_actions
            .push(ProtocolAction::SetTitle { surface_id, title });
    }

    /// Handle xdg_toplevel.set_app_id.
    pub fn set_app_id(&mut self, surface_id: SurfaceId, app_id: String) {
        if let Some(tracker) = self.toplevels.get_mut(&surface_id) {
            tracker.app_id = Some(app_id.clone());
        }
        self.pending_actions
            .push(ProtocolAction::SetAppId { surface_id, app_id });
    }

    /// Handle xdg_surface.ack_configure.
    pub fn ack_configure(&mut self, surface_id: SurfaceId, serial: u32) -> bool {
        if let Some(tracker) = self.toplevels.get_mut(&surface_id) {
            let result = tracker.ack_configure(serial);
            if result {
                self.pending_actions
                    .push(ProtocolAction::AckConfigure { surface_id, serial });
            }
            result
        } else {
            false
        }
    }

    /// Handle wl_surface.commit — maps the window if first commit after configure ack.
    pub fn surface_commit(&mut self, surface_id: SurfaceId) -> bool {
        let mapped = if let Some(tracker) = self.toplevels.get_mut(&surface_id) {
            tracker.map()
        } else {
            false
        };
        self.pending_actions
            .push(ProtocolAction::SurfaceCommit { surface_id });
        mapped
    }

    /// Destroy a surface.
    pub fn destroy_surface(&mut self, surface_id: SurfaceId) {
        self.surface_map.unregister(&surface_id);
        self.toplevels.remove(&surface_id);
        // Remove from client's surface list
        if let Some(client_id) = self.clients.find_by_surface(&surface_id)
            && let Some(client) = self.clients.get_mut(client_id)
        {
            client.remove_surface(&surface_id);
        }
        self.pending_actions
            .push(ProtocolAction::DestroyWindow { surface_id });
    }

    /// Handle xdg_toplevel.set_maximized / unset_maximized.
    pub fn set_maximized(&mut self, surface_id: SurfaceId, maximized: bool) {
        if maximized && let Some(tracker) = self.toplevels.get_mut(&surface_id) {
            let serial = self.serial.next_serial();
            let configure = ToplevelConfigure::maximized(surface_id, &self.output, serial);
            tracker.send_configure(configure);
        }
        self.pending_actions.push(ProtocolAction::SetMaximized {
            surface_id,
            maximized,
        });
    }

    /// Handle xdg_toplevel.set_fullscreen.
    pub fn set_fullscreen(&mut self, surface_id: SurfaceId, fullscreen: bool) {
        if fullscreen && let Some(tracker) = self.toplevels.get_mut(&surface_id) {
            let serial = self.serial.next_serial();
            let configure = ToplevelConfigure {
                surface_id,
                width: self.output.width_px,
                height: self.output.height_px,
                states: vec![XdgToplevelState::Fullscreen, XdgToplevelState::Activated],
                serial,
            };
            tracker.send_configure(configure);
        }
        self.pending_actions.push(ProtocolAction::SetFullscreen {
            surface_id,
            fullscreen,
        });
    }

    /// Handle xdg_toplevel.set_minimized.
    pub fn set_minimized(&mut self, surface_id: SurfaceId) {
        self.pending_actions
            .push(ProtocolAction::SetMinimized { surface_id });
    }

    /// Handle xdg_toplevel.set_min_size / set_max_size.
    pub fn set_size_bounds(
        &mut self,
        surface_id: SurfaceId,
        min_size: Option<(u32, u32)>,
        max_size: Option<(u32, u32)>,
    ) {
        if let Some(tracker) = self.toplevels.get_mut(&surface_id) {
            if min_size.is_some() {
                tracker.min_size = min_size;
            }
            if max_size.is_some() {
                tracker.max_size = max_size;
            }
        }
        self.pending_actions.push(ProtocolAction::SetSizeBounds {
            surface_id,
            min_size,
            max_size,
        });
    }

    /// Handle xdg_toplevel.move request.
    pub fn request_move(&mut self, surface_id: SurfaceId) {
        self.pending_actions
            .push(ProtocolAction::RequestMove { surface_id });
    }

    /// Handle xdg_toplevel.resize request.
    pub fn request_resize(&mut self, surface_id: SurfaceId, edge: u32) {
        self.pending_actions
            .push(ProtocolAction::RequestResize { surface_id, edge });
    }

    /// Route an input event to the appropriate client surface.
    pub fn route_input(&mut self, compositor: &Compositor, event: &InputEvent) {
        let action = compositor.route_input(event);
        match action {
            InputAction::ClientClick(surface_id, x, y) => {
                let serial = self.serial.next_serial();
                let focus_changed =
                    self.pointer_focus
                        .set_focus(Some(surface_id), x as f64, y as f64, serial);
                if focus_changed {
                    // Send pointer enter
                    self.pending_actions.push(ProtocolAction::ForwardPointer {
                        event: WaylandPointerEvent::Enter {
                            surface: surface_id,
                            x: x as f64,
                            y: y as f64,
                        },
                    });
                    // Update keyboard focus too
                    let kb_serial = self.serial.next_serial();
                    self.keyboard_focus.set_focus(Some(surface_id), kb_serial);
                    self.pending_actions.push(ProtocolAction::ForwardKeyboard {
                        event: WaylandKeyboardEvent::Enter {
                            surface: surface_id,
                        },
                    });
                }
                // Forward button event
                if let InputEvent::MouseClick { button, .. } = event {
                    self.pending_actions.push(ProtocolAction::ForwardPointer {
                        event: WaylandPointerEvent::Button {
                            button: *button,
                            x: x as f64,
                            y: y as f64,
                            pressed: true,
                        },
                    });
                }
            }
            InputAction::PointerMove(x, y) => {
                self.pointer_focus.motion(x as f64, y as f64);
                self.pending_actions.push(ProtocolAction::ForwardPointer {
                    event: WaylandPointerEvent::Motion {
                        x: x as f64,
                        y: y as f64,
                    },
                });
            }
            InputAction::KeyToFocused(keycode, modifiers) => {
                let mods = ModifierState::from_raw(modifiers);
                self.keyboard_focus.set_modifiers(mods);
                self.pending_actions.push(ProtocolAction::ForwardKeyboard {
                    event: WaylandKeyboardEvent::Key {
                        keycode,
                        modifiers: mods,
                        pressed: true,
                    },
                });
            }
            _ => {
                // BeginDrag, Close, Minimize, etc. are handled by the compositor directly
            }
        }
    }

    /// Drain all pending protocol actions for processing.
    pub fn drain_actions(&mut self) -> Vec<ProtocolAction> {
        std::mem::take(&mut self.pending_actions)
    }

    /// Apply pending actions to the compositor.
    pub fn apply_actions(&mut self, compositor: &Compositor) -> Vec<ProtocolAction> {
        let actions = self.drain_actions();
        for action in &actions {
            match action {
                ProtocolAction::CreateWindow { title, app_id, .. } => {
                    let t = if title.is_empty() {
                        "Untitled".to_string()
                    } else {
                        title.clone()
                    };
                    let a = if app_id.is_empty() {
                        "unknown".to_string()
                    } else {
                        app_id.clone()
                    };
                    let _ = compositor.create_window(t, a, false);
                }
                ProtocolAction::DestroyWindow { surface_id } => {
                    let _ = compositor.close_window(*surface_id);
                }
                ProtocolAction::SetMaximized {
                    surface_id,
                    maximized,
                } => {
                    if *maximized {
                        let _ = compositor.set_window_state(*surface_id, WindowState::Maximized);
                    } else {
                        let _ = compositor.set_window_state(*surface_id, WindowState::Normal);
                    }
                }
                ProtocolAction::SetFullscreen {
                    surface_id,
                    fullscreen,
                } => {
                    if *fullscreen {
                        let _ = compositor.set_window_state(*surface_id, WindowState::Fullscreen);
                    } else {
                        let _ = compositor.set_window_state(*surface_id, WindowState::Normal);
                    }
                }
                ProtocolAction::SetMinimized { surface_id } => {
                    let _ = compositor.set_window_state(*surface_id, WindowState::Minimized);
                }
                ProtocolAction::SetTitle { .. } => {
                    // Title tracked in XdgToplevelTracker, applied on next render
                }
                ProtocolAction::RequestMove { surface_id } => {
                    compositor.focus_window(*surface_id);
                }
                _ => {}
            }
        }
        actions
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Get the number of tracked surfaces.
    pub fn surface_count(&self) -> usize {
        self.surface_map.len()
    }

    /// Get the number of mapped toplevels.
    pub fn mapped_toplevel_count(&self) -> usize {
        self.toplevels.values().filter(|t| t.mapped).count()
    }
}

impl Default for ProtocolBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps internal [`InputEvent`]s to Wayland protocol actions.
pub fn map_input_to_pointer_event(event: &InputEvent) -> Option<WaylandPointerEvent> {
    match event {
        InputEvent::MouseMove { x, y } => Some(WaylandPointerEvent::Motion {
            x: *x as f64,
            y: *y as f64,
        }),
        InputEvent::MouseClick { button, x, y } => Some(WaylandPointerEvent::Button {
            button: *button,
            x: *x as f64,
            y: *y as f64,
            pressed: true,
        }),
        _ => None,
    }
}

/// Maps internal [`InputEvent`]s to Wayland keyboard protocol actions.
pub fn map_input_to_keyboard_event(event: &InputEvent) -> Option<WaylandKeyboardEvent> {
    match event {
        InputEvent::KeyPress { keycode, modifiers } => Some(WaylandKeyboardEvent::Key {
            keycode: *keycode,
            modifiers: ModifierState::from_raw(*modifiers),
            pressed: true,
        }),
        _ => None,
    }
}

/// Wayland pointer event (protocol-level).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WaylandPointerEvent {
    Motion {
        x: f64,
        y: f64,
    },
    Button {
        button: u32,
        x: f64,
        y: f64,
        pressed: bool,
    },
    Axis {
        horizontal: f64,
        vertical: f64,
    },
    Enter {
        surface: SurfaceId,
        x: f64,
        y: f64,
    },
    Leave {
        surface: SurfaceId,
    },
}

/// Wayland keyboard event (protocol-level).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WaylandKeyboardEvent {
    Key {
        keycode: u32,
        modifiers: ModifierState,
        pressed: bool,
    },
    Enter {
        surface: SurfaceId,
    },
    Leave {
        surface: SurfaceId,
    },
    Modifiers {
        state: ModifierState,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- DataDeviceManager ----

    #[test]
    fn test_data_device_manager_new() {
        let mgr = DataDeviceManager::new();
        assert!(mgr.selections.is_empty());
        assert!(mgr.drag_source.is_none());
    }

    #[test]
    fn test_data_device_manager_default() {
        let mgr = DataDeviceManager::default();
        assert!(mgr.selections.is_empty());
        assert!(mgr.drag_source.is_none());
    }

    #[test]
    fn test_set_and_get_selection() {
        let mut mgr = DataDeviceManager::new();
        let sid = uuid::Uuid::new_v4();
        mgr.set_selection(sid, vec!["text/plain".to_string()], 1);

        let offer = mgr.get_selection(&sid);
        assert!(offer.is_some());
        let offer = offer.unwrap();
        assert_eq!(offer.mime_types, vec!["text/plain"]);
        assert_eq!(offer.source_surface, sid);
        assert_eq!(offer.serial, 1);
    }

    #[test]
    fn test_clear_selection() {
        let mut mgr = DataDeviceManager::new();
        let sid = uuid::Uuid::new_v4();
        mgr.set_selection(sid, vec!["text/plain".to_string()], 1);
        mgr.clear_selection(&sid);
        assert!(mgr.get_selection(&sid).is_none());
    }

    #[test]
    fn test_get_selection_nonexistent() {
        let mgr = DataDeviceManager::new();
        let sid = uuid::Uuid::new_v4();
        assert!(mgr.get_selection(&sid).is_none());
    }

    #[test]
    fn test_start_and_end_drag() {
        let mut mgr = DataDeviceManager::new();
        let source = uuid::Uuid::new_v4();
        let icon = uuid::Uuid::new_v4();
        mgr.start_drag(source, Some(icon), vec!["text/uri-list".to_string()]);

        let drag = mgr.drag_source.as_ref().unwrap();
        assert_eq!(drag.source_surface, source);
        assert_eq!(drag.icon_surface, Some(icon));
        assert!(drag.active);
        assert_eq!(drag.position, (0.0, 0.0));

        mgr.end_drag();
        assert!(mgr.drag_source.is_none());
    }

    #[test]
    fn test_start_drag_no_icon() {
        let mut mgr = DataDeviceManager::new();
        let source = uuid::Uuid::new_v4();
        mgr.start_drag(source, None, vec![]);
        let drag = mgr.drag_source.as_ref().unwrap();
        assert!(drag.icon_surface.is_none());
        assert!(drag.mime_types.is_empty());
    }

    // ---- TextInputState ----

    #[test]
    fn test_text_input_state_new() {
        let state = TextInputState::new();
        assert!(!state.enabled);
        assert!(state.surface_id.is_none());
        assert_eq!(state.content_type, ContentType::Normal);
        assert!(state.surrounding_text.is_empty());
        assert_eq!(state.cursor_position, 0);
        assert!(state.preedit.is_none());
    }

    #[test]
    fn test_text_input_state_default() {
        let state = TextInputState::default();
        assert!(!state.enabled);
    }

    #[test]
    fn test_text_input_enable_disable() {
        let mut state = TextInputState::new();
        let sid = uuid::Uuid::new_v4();
        state.enable(sid);
        assert!(state.enabled);
        assert_eq!(state.surface_id, Some(sid));

        state.disable();
        assert!(!state.enabled);
        assert!(state.surface_id.is_none());
        assert!(state.preedit.is_none());
    }

    #[test]
    fn test_text_input_set_surrounding_text() {
        let mut state = TextInputState::new();
        state.set_surrounding_text("hello world".to_string(), 5);
        assert_eq!(state.surrounding_text, "hello world");
        assert_eq!(state.cursor_position, 5);
    }

    #[test]
    fn test_text_input_commit_preedit() {
        let mut state = TextInputState::new();
        // No preedit -> returns None
        assert!(state.commit_preedit().is_none());

        state.preedit = Some(PreeditState {
            text: "composing".to_string(),
            cursor_begin: 0,
            cursor_end: 9,
        });
        let text = state.commit_preedit();
        assert_eq!(text, Some("composing".to_string()));
        // After commit, preedit is consumed
        assert!(state.preedit.is_none());
    }

    #[test]
    fn test_text_input_clear_preedit() {
        let mut state = TextInputState::new();
        state.preedit = Some(PreeditState {
            text: "test".to_string(),
            cursor_begin: 0,
            cursor_end: 4,
        });
        state.clear_preedit();
        assert!(state.preedit.is_none());
    }

    #[test]
    fn test_text_input_disable_clears_preedit() {
        let mut state = TextInputState::new();
        let sid = uuid::Uuid::new_v4();
        state.enable(sid);
        state.preedit = Some(PreeditState {
            text: "draft".to_string(),
            cursor_begin: 0,
            cursor_end: 5,
        });
        state.disable();
        assert!(state.preedit.is_none());
    }

    // ---- DecorationState ----

    #[test]
    fn test_decoration_state_new() {
        let sid = uuid::Uuid::new_v4();
        let state = DecorationState::new(sid);
        assert_eq!(state.surface_id, sid);
        assert_eq!(state.preferred, DecorationMode::ServerSide);
        assert_eq!(state.current, DecorationMode::ServerSide);
    }

    #[test]
    fn test_decoration_negotiate_server_side() {
        let sid = uuid::Uuid::new_v4();
        let mut state = DecorationState::new(sid);
        state.preferred = DecorationMode::ServerSide;
        let mode = state.negotiate();
        assert_eq!(mode, DecorationMode::ServerSide);
        assert_eq!(state.current, DecorationMode::ServerSide);
    }

    #[test]
    fn test_decoration_negotiate_client_side() {
        let sid = uuid::Uuid::new_v4();
        let mut state = DecorationState::new(sid);
        state.preferred = DecorationMode::ClientSide;
        let mode = state.negotiate();
        assert_eq!(mode, DecorationMode::ClientSide);
        assert_eq!(state.current, DecorationMode::ClientSide);
    }

    #[test]
    fn test_decoration_mode_default() {
        assert_eq!(DecorationMode::default(), DecorationMode::ServerSide);
    }

    // ---- ViewportState ----

    #[test]
    fn test_viewport_state_new() {
        let sid = uuid::Uuid::new_v4();
        let state = ViewportState::new(sid);
        assert_eq!(state.surface_id, sid);
        assert!(state.source.is_none());
        assert!(state.destination.is_none());
    }

    #[test]
    fn test_viewport_set_source() {
        let sid = uuid::Uuid::new_v4();
        let mut state = ViewportState::new(sid);
        state.set_source(10.0, 20.0, 640.0, 480.0);
        let src = state.source.unwrap();
        assert!((src.x - 10.0).abs() < f64::EPSILON);
        assert!((src.y - 20.0).abs() < f64::EPSILON);
        assert!((src.width - 640.0).abs() < f64::EPSILON);
        assert!((src.height - 480.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_viewport_set_destination() {
        let sid = uuid::Uuid::new_v4();
        let mut state = ViewportState::new(sid);
        state.set_destination(800, 600);
        assert_eq!(state.destination, Some((800, 600)));
    }

    #[test]
    fn test_viewport_effective_size_destination_takes_precedence() {
        let sid = uuid::Uuid::new_v4();
        let mut state = ViewportState::new(sid);
        state.set_source(0.0, 0.0, 1920.0, 1080.0);
        state.set_destination(960, 540);
        assert_eq!(state.effective_size(), Some((960, 540)));
    }

    #[test]
    fn test_viewport_effective_size_from_source() {
        let sid = uuid::Uuid::new_v4();
        let mut state = ViewportState::new(sid);
        state.set_source(0.0, 0.0, 1920.0, 1080.0);
        assert_eq!(state.effective_size(), Some((1920, 1080)));
    }

    #[test]
    fn test_viewport_effective_size_none() {
        let sid = uuid::Uuid::new_v4();
        let state = ViewportState::new(sid);
        assert_eq!(state.effective_size(), None);
    }

    // ---- FractionalScale ----

    #[test]
    fn test_fractional_scale_new() {
        let sid = uuid::Uuid::new_v4();
        let fs = FractionalScale::new(sid, 120);
        assert_eq!(fs.surface_id, sid);
        assert_eq!(fs.scale_120, 120);
    }

    #[test]
    fn test_fractional_scale_factor_1x() {
        let sid = uuid::Uuid::new_v4();
        let fs = FractionalScale::new(sid, 120);
        assert!((fs.scale_factor() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fractional_scale_factor_1_25x() {
        let sid = uuid::Uuid::new_v4();
        let fs = FractionalScale::new(sid, 150);
        assert!((fs.scale_factor() - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fractional_scale_factor_2x() {
        let sid = uuid::Uuid::new_v4();
        let fs = FractionalScale::new(sid, 240);
        assert!((fs.scale_factor() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fractional_scale_from_scale() {
        let sid = uuid::Uuid::new_v4();
        let fs = FractionalScale::from_scale(sid, 1.5);
        assert_eq!(fs.scale_120, 180);
        assert!((fs.scale_factor() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fractional_scale_from_scale_2x() {
        let sid = uuid::Uuid::new_v4();
        let fs = FractionalScale::from_scale(sid, 2.0);
        assert_eq!(fs.scale_120, 240);
    }

    // ---- map_input_to_pointer_event ----

    #[test]
    fn test_map_mouse_move_to_pointer() {
        let event = InputEvent::MouseMove { x: 100, y: 200 };
        let result = map_input_to_pointer_event(&event);
        assert!(result.is_some());
        if let Some(WaylandPointerEvent::Motion { x, y }) = result {
            assert!((x - 100.0).abs() < f64::EPSILON);
            assert!((y - 200.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected Motion event");
        }
    }

    #[test]
    fn test_map_mouse_click_to_pointer() {
        let event = InputEvent::MouseClick {
            button: 1,
            x: 50,
            y: 75,
        };
        let result = map_input_to_pointer_event(&event);
        assert!(result.is_some());
        if let Some(WaylandPointerEvent::Button {
            button,
            x,
            y,
            pressed,
        }) = result
        {
            assert_eq!(button, 1);
            assert!((x - 50.0).abs() < f64::EPSILON);
            assert!((y - 75.0).abs() < f64::EPSILON);
            assert!(pressed);
        } else {
            panic!("Expected Button event");
        }
    }

    #[test]
    fn test_map_keypress_to_pointer_returns_none() {
        let event = InputEvent::KeyPress {
            keycode: 42,
            modifiers: 0,
        };
        assert!(map_input_to_pointer_event(&event).is_none());
    }

    // ---- map_input_to_keyboard_event ----

    #[test]
    fn test_map_keypress_to_keyboard() {
        let event = InputEvent::KeyPress {
            keycode: 42,
            modifiers: 0x01, // shift
        };
        let result = map_input_to_keyboard_event(&event);
        assert!(result.is_some());
        if let Some(WaylandKeyboardEvent::Key {
            keycode,
            modifiers,
            pressed,
        }) = result
        {
            assert_eq!(keycode, 42);
            assert!(modifiers.shift);
            assert!(pressed);
        } else {
            panic!("Expected Key event");
        }
    }

    #[test]
    fn test_map_mouse_move_to_keyboard_returns_none() {
        let event = InputEvent::MouseMove { x: 10, y: 20 };
        assert!(map_input_to_keyboard_event(&event).is_none());
    }

    #[test]
    fn test_map_mouse_click_to_keyboard_returns_none() {
        let event = InputEvent::MouseClick {
            button: 1,
            x: 10,
            y: 20,
        };
        assert!(map_input_to_keyboard_event(&event).is_none());
    }

    // ---- ProtocolBridge ----

    #[test]
    fn test_protocol_bridge_new() {
        let bridge = ProtocolBridge::new();
        assert_eq!(bridge.client_count(), 0);
        assert_eq!(bridge.surface_count(), 0);
        assert_eq!(bridge.mapped_toplevel_count(), 0);
    }

    #[test]
    fn test_protocol_bridge_default() {
        let bridge = ProtocolBridge::default();
        assert_eq!(bridge.client_count(), 0);
    }

    #[test]
    fn test_client_connect_no_pid() {
        let mut bridge = ProtocolBridge::new();
        let id = bridge.client_connect(None);
        assert!(id > 0);
        assert_eq!(bridge.client_count(), 1);
        let actions = bridge.drain_actions();
        assert!(actions.iter().any(
            |a| matches!(a, ProtocolAction::ClientConnected { client_id } if *client_id == id)
        ));
    }

    #[test]
    fn test_client_connect_with_pid() {
        let mut bridge = ProtocolBridge::new();
        let id = bridge.client_connect(Some(1234));
        assert!(id > 0);
        assert_eq!(bridge.client_count(), 1);
        let client = bridge.clients.get(id).unwrap();
        assert_eq!(client.pid, Some(1234));
    }

    #[test]
    fn test_client_disconnect() {
        let mut bridge = ProtocolBridge::new();
        let id = bridge.client_connect(None);
        bridge.drain_actions();

        let removed = bridge.client_disconnect(id);
        assert!(removed.is_empty()); // no surfaces
        assert_eq!(bridge.client_count(), 0);

        let actions = bridge.drain_actions();
        assert!(actions.iter().any(
            |a| matches!(a, ProtocolAction::ClientDisconnected { client_id } if *client_id == id)
        ));
    }

    #[test]
    fn test_create_surface() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let result = bridge.create_surface(client_id);
        assert!(result.is_some());
        let (_surface_id, proto_id) = result.unwrap();
        assert!(proto_id > 0);
        assert_eq!(bridge.surface_count(), 1);
    }

    #[test]
    fn test_create_and_destroy_surface() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.drain_actions();

        bridge.destroy_surface(surface_id);
        assert_eq!(bridge.surface_count(), 0);

        let actions = bridge.drain_actions();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, ProtocolAction::DestroyWindow { .. }))
        );
    }

    #[test]
    fn test_create_toplevel_lifecycle() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.drain_actions();

        let configure = bridge.create_toplevel(surface_id, client_id);
        assert_eq!(configure.surface_id, surface_id);
        assert_eq!(configure.width, 0); // initial configure = client picks size
        assert_eq!(configure.height, 0);
        assert!(configure.is_activated());
        let serial = configure.serial;

        let actions = bridge.drain_actions();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, ProtocolAction::CreateWindow { .. }))
        );

        // Ack the configure
        assert!(bridge.ack_configure(surface_id, serial));

        // First commit after ack should map
        assert!(bridge.surface_commit(surface_id));
        assert_eq!(bridge.mapped_toplevel_count(), 1);

        // Second commit should not re-map
        assert!(!bridge.surface_commit(surface_id));
    }

    #[test]
    fn test_ack_configure_wrong_serial() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        assert!(!bridge.ack_configure(surface_id, 99999));
    }

    #[test]
    fn test_ack_configure_no_toplevel() {
        let mut bridge = ProtocolBridge::new();
        let fake_id = uuid::Uuid::new_v4();
        assert!(!bridge.ack_configure(fake_id, 1));
    }

    #[test]
    fn test_set_title() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        bridge.set_title(surface_id, "My Window".to_string());
        let tracker = bridge.toplevels.get(&surface_id).unwrap();
        assert_eq!(tracker.title, Some("My Window".to_string()));

        let actions = bridge.drain_actions();
        assert!(
            actions.iter().any(
                |a| matches!(a, ProtocolAction::SetTitle { title, .. } if title == "My Window")
            )
        );
    }

    #[test]
    fn test_set_app_id() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        bridge.set_app_id(surface_id, "org.example.app".to_string());
        let tracker = bridge.toplevels.get(&surface_id).unwrap();
        assert_eq!(tracker.app_id, Some("org.example.app".to_string()));
    }

    #[test]
    fn test_set_minimized() {
        let mut bridge = ProtocolBridge::new();
        let sid = uuid::Uuid::new_v4();
        bridge.set_minimized(sid);
        let actions = bridge.drain_actions();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, ProtocolAction::SetMinimized { .. }))
        );
    }

    #[test]
    fn test_request_move_and_resize() {
        let mut bridge = ProtocolBridge::new();
        let sid = uuid::Uuid::new_v4();

        bridge.request_move(sid);
        bridge.request_resize(sid, 2);

        let actions = bridge.drain_actions();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, ProtocolAction::RequestMove { .. }))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, ProtocolAction::RequestResize { edge, .. } if *edge == 2))
        );
    }

    #[test]
    fn test_set_size_bounds() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        bridge.set_size_bounds(surface_id, Some((100, 100)), Some((800, 600)));
        let tracker = bridge.toplevels.get(&surface_id).unwrap();
        assert_eq!(tracker.min_size, Some((100, 100)));
        assert_eq!(tracker.max_size, Some((800, 600)));
    }

    #[test]
    fn test_set_size_bounds_none_does_not_overwrite() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        bridge.set_size_bounds(surface_id, Some((50, 50)), None);
        bridge.set_size_bounds(surface_id, None, Some((400, 400)));
        let tracker = bridge.toplevels.get(&surface_id).unwrap();
        assert_eq!(tracker.min_size, Some((50, 50)));
        assert_eq!(tracker.max_size, Some((400, 400)));
    }

    #[test]
    fn test_set_maximized() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        bridge.set_maximized(surface_id, true);
        let actions = bridge.drain_actions();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, ProtocolAction::SetMaximized { maximized, .. } if *maximized))
        );
    }

    #[test]
    fn test_set_fullscreen() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        bridge.set_fullscreen(surface_id, true);
        let actions = bridge.drain_actions();
        assert!(
            actions.iter().any(
                |a| matches!(a, ProtocolAction::SetFullscreen { fullscreen, .. } if *fullscreen)
            )
        );
    }

    #[test]
    fn test_client_disconnect_removes_surfaces_and_clears_focus() {
        let mut bridge = ProtocolBridge::new();
        let client_id = bridge.client_connect(None);
        let (surface_id, _) = bridge.create_surface(client_id).unwrap();
        bridge.create_toplevel(surface_id, client_id);
        bridge.drain_actions();

        // Set focus to this surface
        bridge
            .pointer_focus
            .set_focus(Some(surface_id), 0.0, 0.0, 1);
        bridge.keyboard_focus.set_focus(Some(surface_id), 1);

        let removed = bridge.client_disconnect(client_id);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], surface_id);
        assert!(bridge.pointer_focus.surface_id.is_none());
        assert!(bridge.keyboard_focus.surface_id.is_none());
        assert_eq!(bridge.surface_count(), 0);
        assert!(bridge.toplevels.is_empty());
    }

    #[test]
    fn test_client_disconnect_nonexistent() {
        let mut bridge = ProtocolBridge::new();
        let removed = bridge.client_disconnect(999);
        assert!(removed.is_empty());
    }

    #[test]
    fn test_drain_actions_empties_queue() {
        let mut bridge = ProtocolBridge::new();
        bridge.client_connect(None);
        let first = bridge.drain_actions();
        assert!(!first.is_empty());
        let second = bridge.drain_actions();
        assert!(second.is_empty());
    }

    #[test]
    fn test_surface_commit_no_toplevel() {
        let mut bridge = ProtocolBridge::new();
        let fake = uuid::Uuid::new_v4();
        // Should not panic, just returns false
        assert!(!bridge.surface_commit(fake));
    }

    // ---- ContentType ----

    #[test]
    fn test_content_type_default() {
        assert_eq!(ContentType::default(), ContentType::Normal);
    }
}
