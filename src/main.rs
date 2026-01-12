mod circle;
mod ffmpeg;
mod onvif;
mod render;

use ab_glyph::FontArc;
use circle::CircleState;
use render::render_frame;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::time::{Duration, sleep};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::onvif::{CameraState, Preset, SharedCameraState};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn create_default_presets() -> Vec<Preset> {
    (1..=10)
        .map(|i| Preset {
            token: i.to_string(),
            name: format!("Preset {}", i),
            pan: 0.0,
            tilt: 0.0,
            zoom: 1.0,
        })
        .collect()
}

#[tokio::main]
async fn main() {
    let width = 480;
    let height = 320;
    let fps = 15;

    let rtsp_url = std::env::var("RTSP_URL").unwrap_or("rtsp://127.0.0.1:8554/cam1".into());

    let onvif_state: SharedCameraState = Arc::new(Mutex::new(CameraState {
        pan: 0.0,
        tilt: 0.0,
        zoom: 1.0,
        presets: create_default_presets(),
    }));

    let http_state = onvif_state.clone();
    let stream_state = onvif_state.clone();

    init_tracing();

    tokio::spawn(async move {
        onvif::start_http_server(http_state).await;
    });

    run_stream_supervisor(rtsp_url, width, height, fps, stream_state).await;
}

async fn run_stream_supervisor(
    rtsp_url: String,
    width: u32,
    height: u32,
    fps: u32,
    onvif_state: SharedCameraState,
) {
    let retry_delay = Duration::from_secs(5);
    let mut state: CircleState = CircleState::new(width, height);
    let font_data = include_bytes!("../assets/FreeMono.ttf");
    let font = FontArc::try_from_slice(font_data).expect("invalid font");

    eprintln!("Starting ffmpeg -> {}", rtsp_url);

    loop {
        // eprintln!("Starting ffmpeg -> {}", rtsp_url);

        let mut ffmpeg = match ffmpeg::start_ffmpeg(&rtsp_url, width, height, fps).await {
            Ok(p) => p,
            Err(_e) => {
                // eprintln!("Failed to start ffmpeg: {_e}");
                sleep(retry_delay).await;
                continue;
            }
        };

        loop {
            state.update();
            let frame = render_frame(&state, &onvif_state, width, height, &font);

            if let Err(_e) = ffmpeg.stdin.write_all(frame.as_raw()).await {
                // eprintln!("ffmpeg stream error: {_e}");
                break;
            }

            sleep(Duration::from_millis(1000 / fps as u64)).await;
        }

        let _ = ffmpeg.child.kill().await;

        // eprintln!("Reconnecting in {}s...", retry_delay.as_secs());
        sleep(retry_delay).await;
    }
}
