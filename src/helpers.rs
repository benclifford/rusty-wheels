use crate::structs::FrameState;
use palette::encoding::pixel::Pixel;
use palette::Hsv;
use palette::Srgb;

/// Turns spin position into a saturated rainbow wheel
pub fn spinpos_to_rgb(framestate: &FrameState) -> (u8, u8, u8) {
    fraction_to_rgb(framestate.spin_pos, None)
}

/// turns a value from 0..1 into RGB
pub fn fraction_to_rgb(fraction: f32, value: Option<f32>) -> (u8, u8, u8) {
    let hue = (fraction * 360.0).min(360.0);

    let real_value = match value {
        Some(v) => v,
        None => 0.2,
    };

    let hsv: Hsv = Hsv::from_components((hue, 1.0, real_value));

    let srgb = Srgb::from(hsv);

    let pixels: [u8; 3] = srgb.into_linear().into_format().into_raw();

    let [red, green, blue] = pixels;

    (red, green, blue)
}
