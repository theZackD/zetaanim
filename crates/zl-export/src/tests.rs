use super::*;
use std::process::{Command, Stdio};

#[test]
fn frame_count_covers_the_start_and_the_end() {
    assert_eq!(frame_count(6.0, 60), 361);
    assert_eq!(frame_count(1.0, 24), 25);
    assert_eq!(frame_count(0.5, 30), 16);
}

#[test]
fn rounding_errors_do_not_add_a_frame() {
    assert_eq!(frame_count(0.1 + 0.2, 60), 19);
    assert_eq!(frame_count(1.1 + 2.2, 60), 199);
}

#[test]
fn partial_last_interval_still_gets_its_frame() {
    assert_eq!(frame_count(1.0 / 60.0 + 0.001, 60), 3);
}

#[test]
fn invalid_or_empty_durations_give_a_single_frame() {
    for duration in [0.0, -0.0, -2.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(frame_count(duration, 60), 1, "duration {duration}");
    }
    assert_eq!(frame_count(6.0, 0), 1);
}

#[test]
fn huge_duration_does_not_overflow() {
    assert_eq!(frame_count(1e300, 60), u64::MAX);
}

#[test]
fn frame_time_divides_and_stops_at_the_end() {
    assert_eq!(frame_time(0, 60, 6.0), 0.0);
    assert_eq!(frame_time(30, 60, 6.0), 0.5);
    assert_eq!(frame_time(360, 60, 6.0), 6.0);
    assert_eq!(frame_time(18, 60, 0.1 + 0.2), 0.1 + 0.2);
    let duration = 0.26;
    let last = frame_count(duration, 60) - 1;
    assert_eq!(last, 16);
    assert_eq!(frame_time(last, 60, duration), duration);
}

#[test]
fn frame_times_do_not_drift_over_long_videos() {
    let mut accumulated = 0.0;
    for _ in 0..36_000 {
        accumulated += 1.0 / 60.0;
    }
    assert_ne!(accumulated, 600.0);
    assert_eq!(frame_time(36_000, 60, 1000.0), 600.0);
}

const MISSING_FFMPEG: &str = "/nonexistent/ffmpeg";

fn tools_available() -> bool {
    ["ffmpeg", "ffprobe"].iter().all(|tool| {
        Command::new(tool)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    })
}

fn skip() {
    eprintln!("ffmpeg or ffprobe not found: test skipped");
}

fn temp_video(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("zl-export-{}-{name}.mp4", std::process::id()))
}

fn red_frame(width: usize, height: usize) -> Vec<u8> {
    [255, 0, 0, 255].repeat(width * height)
}

fn probe(path: &Path) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-count_frames",
            "-show_entries",
            "stream=width,height,nb_read_frames,color_range,color_space,color_transfer,color_primaries",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn first_luma(path: &Path) -> u8 {
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "yuv420p",
            "-",
        ])
        .output()
        .unwrap();
    output.stdout[0]
}

#[test]
fn invalid_dimensions_and_frame_rate_are_refused_before_ffmpeg_starts() {
    for (width, height, fps) in [
        (63, 36, 10),
        (64, 35, 10),
        (0, 36, 10),
        (64, 0, 10),
        (8194, 36, 10),
        (64, 8194, 10),
        (64, 36, 0),
    ] {
        let error = Video::open(
            Path::new(MISSING_FFMPEG),
            &temp_video("refused"),
            width,
            height,
            fps,
        )
        .err()
        .unwrap();
        assert_eq!(
            error.kind(),
            ErrorKind::InvalidInput,
            "{width}x{height} at {fps}"
        );
    }
}

#[test]
fn largest_dimensions_pass_the_checks() {
    let error = Video::open(
        Path::new(MISSING_FFMPEG),
        &temp_video("largest"),
        8192,
        8192,
        60,
    )
    .err()
    .unwrap();
    assert_eq!(error.kind(), ErrorKind::NotFound);
}

#[test]
fn missing_ffmpeg_gives_a_clear_error() {
    let error = Video::open(
        Path::new(MISSING_FFMPEG),
        &temp_video("missing"),
        64,
        36,
        10,
    )
    .err()
    .unwrap();
    assert_eq!(error.kind(), ErrorKind::NotFound);
    let message = error.to_string();
    assert!(message.contains(MISSING_FFMPEG), "{message}");
    assert!(message.contains("brew install ffmpeg"), "{message}");
}

#[test]
fn video_has_its_frames_size_and_bt709_colors() {
    if !tools_available() {
        return skip();
    }
    let path = temp_video("red");
    let mut video = Video::open(Path::new("ffmpeg"), &path, 64, 36, 10).unwrap();
    for _ in 0..10 {
        video.write_frame(&red_frame(64, 36)).unwrap();
    }
    video.finish().unwrap();
    assert!(!part_of(&path).exists());
    assert_eq!(probe(&path), "64,36,tv,bt709,bt709,bt709,10");
    let luma = first_luma(&path);
    assert!(
        (61..=65).contains(&luma),
        "red luma {luma}, BT.601 gives 81"
    );
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn frame_of_the_wrong_size_is_refused() {
    if !tools_available() {
        return skip();
    }
    let path = temp_video("wrong");
    let mut video = Video::open(Path::new("ffmpeg"), &path, 64, 36, 10).unwrap();
    let frame = red_frame(64, 36);
    for bytes in [&frame[..10], &[frame.as_slice(), &[0]].concat()[..]] {
        let error = video.write_frame(bytes).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }
    drop(video);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn abandoned_video_stops_ffmpeg() {
    if !tools_available() {
        return skip();
    }
    let path = temp_video("abandoned");
    let mut video = Video::open(Path::new("ffmpeg"), &path, 64, 36, 10).unwrap();
    video.write_frame(&red_frame(64, 36)).unwrap();
    let pid = video.child.as_ref().unwrap().id().to_string();
    drop(video);
    let still_running = Command::new("kill")
        .args(["-0", &pid])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success();
    assert!(!still_running, "ffmpeg {pid} is still running");
    assert!(!path.exists() && !part_of(&path).exists());
}

#[test]
fn last_frame_shows_exactly_the_end_of_the_timeline() {
    let duration = 2.2 + 0.2 + 0.4 + 0.2;
    assert_ne!(duration, 3.0);
    let last = frame_count(duration, 60) - 1;
    assert_eq!(last, 180);
    assert_eq!(frame_time(last, 60, duration), duration);
    assert_eq!(frame_time(last - 1, 60, duration), 179.0 / 60.0);
    assert_eq!(frame_time(last + 5, 60, duration), duration);
}

#[test]
fn invalid_durations_show_the_start() {
    for duration in [0.0, -2.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(frame_time(0, 60, duration), 0.0, "duration {duration}");
    }
}

fn part_of(path: &Path) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}.part", path.display()))
}

fn first_error(video: Video, frame: &[u8]) -> io::Error {
    let mut video = video;
    for _ in 0..5 {
        if let Err(error) = video.write_frame(frame) {
            return error;
        }
    }
    video.finish().unwrap_err()
}

#[test]
fn ffmpeg_failure_is_reported_with_its_message_however_it_shows() {
    if !tools_available() {
        return skip();
    }
    let path = std::env::temp_dir()
        .join(format!("zl-export-{}-missing-dir", std::process::id()))
        .join("video.mp4");
    for (width, height) in [(64, 36), (1920, 1080)] {
        let video = Video::open(Path::new("ffmpeg"), &path, width, height, 10).unwrap();
        let message = first_error(video, &red_frame(width as usize, height as usize)).to_string();
        assert!(
            message.contains("missing-dir"),
            "{width}x{height}: {message}"
        );
    }
}

#[test]
fn finished_video_replaces_the_previous_one_and_leaves_no_part() {
    if !tools_available() {
        return skip();
    }
    let path = temp_video("replaced");
    for frames in [10, 4] {
        let mut video = Video::open(Path::new("ffmpeg"), &path, 64, 36, 10).unwrap();
        for _ in 0..frames {
            video.write_frame(&red_frame(64, 36)).unwrap();
        }
        video.finish().unwrap();
        assert!(!part_of(&path).exists());
    }
    assert!(probe(&path).ends_with(",4"), "{}", probe(&path));
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn abandoned_export_keeps_the_previous_video() {
    if !tools_available() {
        return skip();
    }
    let path = temp_video("kept");
    let mut video = Video::open(Path::new("ffmpeg"), &path, 64, 36, 10).unwrap();
    for _ in 0..10 {
        video.write_frame(&red_frame(64, 36)).unwrap();
    }
    video.finish().unwrap();
    let before = std::fs::read(&path).unwrap();
    let mut abandoned = Video::open(Path::new("ffmpeg"), &path, 64, 36, 10).unwrap();
    for _ in 0..30 {
        abandoned.write_frame(&red_frame(64, 36)).unwrap();
    }
    drop(abandoned);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!part_of(&path).exists());
    std::fs::remove_file(&path).unwrap();
}
