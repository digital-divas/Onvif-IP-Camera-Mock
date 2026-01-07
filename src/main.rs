mod circle;
mod ffmpeg;
mod render;

use ab_glyph::FontArc;
use circle::CircleState;
use render::render_frame;
use tokio::io::AsyncWriteExt;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let width = 480;
    let height = 320;
    let fps = 15;

    let rtsp_url = std::env::var("RTSP_URL").unwrap_or("rtsp://127.0.0.1:8554/cam1".into());

    run_stream_supervisor(rtsp_url, width, height, fps).await;
}

async fn run_stream_supervisor(rtsp_url: String, width: u32, height: u32, fps: u32) {
    let retry_delay = Duration::from_secs(5);
    let mut state: CircleState = CircleState::new(width, height);
    let font_data = include_bytes!("../assets/FreeMono.ttf");
    let font = FontArc::try_from_slice(font_data).expect("invalid font");

    loop {
        eprintln!("Starting ffmpeg -> {}", rtsp_url);

        let mut ffmpeg = match ffmpeg::start_ffmpeg(&rtsp_url, width, height, fps).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to start ffmpeg: {e}");
                sleep(retry_delay).await;
                continue;
            }
        };

        loop {
            state.update();
            let frame = render_frame(&state, width, height, &font);

            if let Err(e) = ffmpeg.stdin.write_all(frame.as_raw()).await {
                eprintln!("ffmpeg stream error: {e}");
                break;
            }

            sleep(Duration::from_millis(1000 / fps as u64)).await;
        }

        let _ = ffmpeg.child.kill().await;

        eprintln!("Reconnecting in {}s...", retry_delay.as_secs());
        sleep(retry_delay).await;
    }
}
