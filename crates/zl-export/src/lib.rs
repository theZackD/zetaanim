use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread::{self, JoinHandle};

const FRAME_TOLERANCE: f64 = 1e-6;
const MAX_DIMENSION: u32 = 8192;
const COLOR_FILTER: &str = "scale=out_color_matrix=bt709:out_range=tv,format=yuv420p,setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709:range=tv";

pub struct Video {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    frame_size: usize,
    errors: Option<JoinHandle<String>>,
}

impl Video {
    pub fn open(
        ffmpeg: &Path,
        path: &Path,
        width: u32,
        height: u32,
        fps: u32,
    ) -> io::Result<Video> {
        if !(width.is_multiple_of(2)
            && height.is_multiple_of(2)
            && width > 0
            && height > 0
            && width <= MAX_DIMENSION
            && height <= MAX_DIMENSION)
        {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "video dimensions must be even, non-zero and at most 8192",
            ));
        }
        if fps == 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "frame rate must be positive",
            ));
        }
        let size = format!("{width}x{height}");
        let rate = fps.to_string();
        let mut child = Command::new(ffmpeg)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgba",
                "-video_size",
                size.as_str(),
                "-framerate",
                rate.as_str(),
                "-i",
                "-",
                "-vf",
                COLOR_FILTER,
                "-c:v",
                "libx264",
                "-crf",
                "18",
                "-movflags",
                "+faststart",
                "-y",
            ])
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                if error.kind() == ErrorKind::NotFound {
                    io::Error::new(
                        ErrorKind::NotFound,
                        format!(
                            "ffmpeg not found at {}: install it with `brew install ffmpeg`",
                            ffmpeg.display()
                        ),
                    )
                } else {
                    error
                }
            })?;
        let stdin = child.stdin.take();
        let stderr = child.stderr.take();
        let errors = thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut stderr) = stderr {
                let _ = stderr.read_to_string(&mut text);
            }
            text
        });
        Ok(Video {
            child: Some(child),
            stdin,
            errors: Some(errors),
            frame_size: width as usize * height as usize * 4,
        })
    }

    pub fn write_frame(&mut self, rgba: &[u8]) -> io::Result<()> {
        if rgba.len() != self.frame_size {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "a frame must have {} bytes, got {}",
                    self.frame_size,
                    rgba.len()
                ),
            ));
        }
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(io::Error::new(
                ErrorKind::BrokenPipe,
                "ffmpeg input is closed",
            ));
        };
        stdin.write_all(rgba)
    }

    pub fn finish(mut self) -> io::Result<()> {
        drop(self.stdin.take());
        let Some(mut child) = self.child.take() else {
            return Err(io::Error::other("ffmpeg is not running"));
        };
        let status = child.wait()?;
        let errors = self
            .errors
            .take()
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default();
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "ffmpeg failed ({status}): {}",
                errors.trim()
            )))
        }
    }
}

impl Drop for Video {
    fn drop(&mut self) {
        drop(self.stdin.take());
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(handle) = self.errors.take() {
            let _ = handle.join();
        }
    }
}

pub fn frame_count(duration: f64, fps: u32) -> u64 {
    if !(duration.is_finite() && duration > 0.0) {
        1
    } else {
        ((duration * f64::from(fps) - FRAME_TOLERANCE).ceil() as u64).saturating_add(1)
    }
}

pub fn frame_time(index: u64, fps: u32, duration: f64) -> f64 {
    (index as f64 / f64::from(fps)).min(duration)
}

#[cfg(test)]
mod tests;
