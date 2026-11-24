use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};

const FRAME_TOLERANCE: f64 = 1e-6;
const MAX_DIMENSION: u32 = 8192;
const COLOR_FILTER: &str = "scale=out_color_matrix=bt709:out_range=tv,format=yuv420p,setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709:range=tv";

pub struct Video {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    frame_size: usize,
    errors: Option<JoinHandle<String>>,
    path: PathBuf,
    part: PathBuf,
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
        let mut part = path.as_os_str().to_owned();
        part.push(".part");
        let part = PathBuf::from(part);
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
                "-f",
                "mp4",
                "-y",
            ])
            .arg(&part)
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
            path: path.to_path_buf(),
            part,
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
        if let Err(error) = stdin.write_all(rgba) {
            let (_, message) = self.close();
            if message.is_empty() {
                return Err(error);
            }
            return Err(io::Error::new(
                error.kind(),
                format!("{error}; ffmpeg: {message}"),
            ));
        }
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<()> {
        let (status, message) = self.close();
        match status {
            Some(status) if status.success() => fs::rename(&self.part, &self.path),
            Some(status) => Err(io::Error::other(format!(
                "ffmpeg failed ({status}): {message}"
            ))),
            None => Err(io::Error::other(format!(
                "ffmpeg had already stopped: {message}"
            ))),
        }
    }

    fn close(&mut self) -> (Option<ExitStatus>, String) {
        drop(self.stdin.take());
        let status = self.child.take().and_then(|mut child| child.wait().ok());
        let message = self
            .errors
            .take()
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default();
        (status, message.trim().to_string())
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
        let _ = fs::remove_file(&self.part);
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
    if !(duration.is_finite() && duration > 0.0) {
        return 0.0;
    }
    if index.saturating_add(1) >= frame_count(duration, fps) {
        return duration;
    }
    index as f64 / f64::from(fps)
}

#[cfg(test)]
mod tests;
