use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct FfmpegProcess {
    pub child: Child,
    pub stdin: tokio::process::ChildStdin,
}

pub async fn start_ffmpeg(
    rtsp_url: &str,
    width: u32,
    height: u32,
    fps: u32,
) -> anyhow::Result<FfmpegProcess> {
    let size = format!("{}x{}", width, height);
    let fps = fps.to_string();

    let mut cmd = Command::new("ffmpeg");

    cmd.args([
        "-loglevel",
        "warning",
        // input raw frames
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-s",
        &size,
        "-r",
        &fps,
        "-i",
        "-",
        // encoder
        "-c:v",
        "libx264",
        "-preset",
        "ultrafast",
        "-tune",
        "zerolatency",
        "-pix_fmt",
        "yuv420p",
        // RTSP
        "-f",
        "rtsp",
        "-rtsp_transport",
        "tcp",
        rtsp_url,
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::null());

    let mut child = cmd.spawn()?;

    let stdin = child.stdin.take().expect("ffmpeg stdin");

    Ok(FfmpegProcess { child, stdin })
}
