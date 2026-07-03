use aethersafha::{
    DamageTracker, DesktopRenderer, Framebuffer, Rectangle,
    renderer::{COLOR_BLACK, COLOR_WHITE, argb, blend, decompose, text_width},
};

#[test]
fn framebuffer_roundtrip() {
    let fill = argb(255, 100, 150, 200);
    let fb = Framebuffer::new(64, 64, fill).unwrap();
    assert_eq!(fb.width, 64);
    assert_eq!(fb.height, 64);
    assert_eq!(fb.get(0, 0), Some(fill));
    assert_eq!(fb.get(63, 63), Some(fill));
    assert_eq!(fb.get(64, 0), None);
}

#[test]
fn framebuffer_zero_dimension_is_empty() {
    let fb = Framebuffer::new(0, 100, 0).unwrap();
    assert_eq!(fb.pixels.len(), 0);
    assert_eq!(fb.get(0, 0), None);
}

#[test]
fn blend_opaque_over_anything() {
    let src = argb(255, 100, 100, 100);
    let dst = argb(255, 200, 200, 200);
    assert_eq!(blend(src, dst), src);
}

#[test]
fn blend_transparent_is_identity() {
    let src = argb(0, 100, 100, 100);
    let dst = argb(255, 200, 200, 200);
    assert_eq!(blend(src, dst), dst);
}

#[test]
fn decompose_roundtrip() {
    let pixel = argb(10, 20, 30, 40);
    let (a, r, g, b) = decompose(pixel);
    assert_eq!((a, r, g, b), (10, 20, 30, 40));
}

#[test]
fn blit_opaque_overwrites() {
    let mut dst = Framebuffer::new(10, 10, COLOR_BLACK).unwrap();
    let src = Framebuffer::new(5, 5, COLOR_WHITE).unwrap();
    dst.blit(&src, 0, 0);

    // Top-left 5x5 should be white
    assert_eq!(dst.get(0, 0), Some(COLOR_WHITE));
    assert_eq!(dst.get(4, 4), Some(COLOR_WHITE));
    // Outside blit area should be black
    assert_eq!(dst.get(5, 5), Some(COLOR_BLACK));
}

#[test]
fn blit_with_negative_offset_clips() {
    let mut dst = Framebuffer::new(10, 10, COLOR_BLACK).unwrap();
    let src = Framebuffer::new(5, 5, COLOR_WHITE).unwrap();
    dst.blit(&src, -2, -2);

    // Only the visible portion should be blitted
    assert_eq!(dst.get(0, 0), Some(COLOR_WHITE));
    assert_eq!(dst.get(2, 2), Some(COLOR_WHITE));
    assert_eq!(dst.get(3, 3), Some(COLOR_BLACK));
}

#[test]
fn fill_rect_clips_to_framebuffer() {
    let mut fb = Framebuffer::new(10, 10, COLOR_BLACK).unwrap();
    fb.fill_rect(
        &Rectangle {
            x: -5,
            y: -5,
            width: 20,
            height: 20,
        },
        COLOR_WHITE,
    );
    // Entire framebuffer should be white (rect extends beyond bounds)
    assert_eq!(fb.get(0, 0), Some(COLOR_WHITE));
    assert_eq!(fb.get(9, 9), Some(COLOR_WHITE));
}

#[test]
fn damage_tracker_lifecycle() {
    let mut tracker = DamageTracker::new(1920, 1080);
    assert!(tracker.has_damage()); // First frame is full damage

    let bounds = tracker.flush();
    assert_eq!(bounds.width, 1920);
    assert_eq!(bounds.height, 1080);
    assert!(!tracker.has_damage());

    tracker.add_damage(Rectangle {
        x: 10,
        y: 10,
        width: 100,
        height: 100,
    });
    assert!(tracker.has_damage());
    assert_eq!(tracker.region_count(), 1);

    let bounds = tracker.flush();
    assert_eq!(bounds.x, 10);
    assert_eq!(bounds.y, 10);
}

#[test]
fn text_width_calculation() {
    assert_eq!(text_width(""), 0);
    assert_eq!(text_width("A"), 5); // 5px glyph, no trailing gap
    assert_eq!(text_width("AB"), 11); // 5 + 1 + 5
}

#[test]
fn desktop_renderer_creates() {
    let renderer = DesktopRenderer::new(1920, 1080);
    assert!(renderer.is_some());
}

#[test]
fn as_bytes_length() {
    let fb = Framebuffer::new(10, 10, COLOR_BLACK).unwrap();
    let bytes = fb.as_bytes();
    assert_eq!(bytes.len(), 10 * 10 * 4);
}
