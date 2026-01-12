use ab_glyph::{FontArc, PxScale};
use chrono::Utc;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_text_mut};

use crate::{circle::CircleState, onvif::SharedCameraState};

pub fn render_frame(
    state: &CircleState,
    onvif_state: &SharedCameraState,
    width: u32,
    height: u32,
    font: &FontArc,
) -> RgbImage {
    let mut img = RgbImage::from_pixel(width, height, Rgb([0, 0, 0]));

    draw_filled_circle_mut(
        &mut img,
        (state.x as i32, state.y as i32),
        state.radius,
        state.color,
    );

    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let scale = PxScale::from(16.0);

    draw_text_mut(
        &mut img,
        Rgb([255, 255, 255]),
        10,
        10,
        scale,
        font,
        &timestamp,
    );

    let (pan, tilt, zoom) = {
        let s = onvif_state.lock().unwrap();
        (s.pan, s.tilt, s.zoom)
    };

    let ptz_position = format!("pan={} tilt={} zoom={}", pan, tilt, zoom);

    draw_text_mut(
        &mut img,
        Rgb([255, 255, 255]),
        10,
        (height - 26).try_into().unwrap(),
        scale,
        font,
        &ptz_position,
    );

    img
}
