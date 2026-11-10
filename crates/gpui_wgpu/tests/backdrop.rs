//! A dialog's dismissal scales its painted subtree; the backdrop blur under it
//! must scale with it while keeping its blur radius. The blur passes
//! themselves are unit tests in `src/backdrop.rs`.

#[test]
fn scaled_dialog_keeps_its_center_and_blur_radius() {
    use gpui::{Bounds, ContentMask, Corners, ScaledPixels, Scene, point, size};
    let bounds = Bounds::new(
        point(ScaledPixels(100.0), ScaledPixels(50.0)),
        size(ScaledPixels(400.0), ScaledPixels(200.0)),
    );
    let mut original = Scene::default();
    original.push_layer(bounds);
    original.insert_backdrop_blur(gpui::BackdropBlur {
        order: 0,
        opacity: 0.5,
        blur_radius: ScaledPixels(44.0),
        bounds,
        content_mask: ContentMask { bounds },
        corner_radii: Corners::all(ScaledPixels(12.0)),
    });
    original.insert_primitive(gpui::Quad {
        bounds,
        content_mask: ContentMask { bounds },
        ..Default::default()
    });
    original.pop_layer();
    let mut scaled = Scene::default();
    scaled.append_scaled(&original, bounds.center(), 0.96);
    let blur = &scaled.backdrop_blurs[0];
    assert_eq!(blur.bounds.center(), bounds.center());
    assert_eq!(blur.bounds.size.width, ScaledPixels(384.0));
    assert_eq!(blur.bounds.size.height, ScaledPixels(192.0));
    assert_eq!(blur.blur_radius, ScaledPixels(44.0));
    assert_eq!(blur.opacity, 0.5);
    assert_eq!(scaled.quads[0].bounds, blur.bounds);
}
