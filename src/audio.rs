//! Original, optional PCM cues. No assets, device-volume changes or UI-thread playback.
//!
//! Integration: keep one player; call `play` only for new events, passing 0.0
//! when muted/quiet. Construction and silent calls start no thread. Positive
//! volume lazily starts one bounded worker on Windows. Other OSes are silent.
//! Dropping the player abandons pending cues without joining; a synchronous
//! cue already in flight may finish (at most 220 ms of authored audio, excluding
//! driver delays). Muting does not retroactively stop an already submitted cue.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioCue {
    Arrival,
    Reward,
    Equip,
}

/// Best-effort sound: unavailable devices and a full queue are silent failures.
/// Own one instance in the app, not one per frame/event.
#[derive(Default)]
pub struct AudioPlayer {
    #[cfg(windows)]
    worker: Option<Worker>,
    #[cfg(windows)]
    attempted: bool,
}

impl AudioPlayer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Nonblocking enqueue, never synthesis or playback on the caller thread.
    /// Finite volume is clamped to 0..=1; zero/negative/nonfinite is a no-op.
    /// At most one waiting cue is retained; stale cues (>250 ms) are discarded.
    pub fn play(&mut self, cue: AudioCue, volume: f32) {
        let volume = normalize_volume(volume);
        if volume == 0.0 {
            return;
        }
        #[cfg(windows)]
        {
            if !self.attempted {
                self.attempted = true;
                self.worker = Worker::spawn(play_wave);
            }
            if let Some(worker) = &self.worker {
                worker.enqueue(cue, volume);
            }
        }
        #[cfg(not(windows))]
        let _ = cue;
    }
}

fn normalize_volume(volume: f32) -> f32 {
    if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(any(windows, test))]
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    time::{Duration, Instant},
};

#[cfg(any(windows, test))]
const SAMPLE_RATE: u32 = 22_050;
#[cfg(any(windows, test))]
const MAX_CUE_AGE: Duration = Duration::from_millis(250);

#[cfg(any(windows, test))]
struct Request {
    cue: AudioCue,
    volume: f32,
    created: Instant,
}

#[cfg(any(windows, test))]
struct Worker {
    sender: SyncSender<Request>,
    stopped: Arc<AtomicBool>,
}

#[cfg(any(windows, test))]
impl Worker {
    fn spawn(playback: impl FnMut(&[u8]) + Send + 'static) -> Option<Self> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let stopped = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stopped);
        // Detach intentionally: a slow device driver must never block shutdown.
        let handle = std::thread::Builder::new()
            .name("cpu-express-audio".into())
            .spawn(move || run_worker(receiver, thread_stop, playback))
            .ok()?;
        drop(handle);
        Some(Self { sender, stopped })
    }

    fn enqueue(&self, cue: AudioCue, volume: f32) {
        let _ = self.sender.try_send(Request {
            cue,
            volume,
            created: Instant::now(),
        });
    }
}

#[cfg(any(windows, test))]
impl Drop for Worker {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        // Dropping sender wakes recv; the worker checks stop before the next cue.
    }
}

#[cfg(any(windows, test))]
fn run_worker(
    receiver: Receiver<Request>,
    stopped: Arc<AtomicBool>,
    mut playback: impl FnMut(&[u8]),
) {
    while let Ok(request) = receiver.recv() {
        if stopped.load(Ordering::Acquire) {
            break;
        }
        if request.created.elapsed() > MAX_CUE_AGE {
            continue;
        }
        let volume = normalize_volume(request.volume);
        if volume == 0.0 {
            continue;
        }
        let wave = synthesize(request.cue, volume);
        if !stopped.load(Ordering::Acquire) && request.created.elapsed() <= MAX_CUE_AGE {
            playback(&wave);
        }
    }
}

/// RIFF/WAVE, mono signed 16-bit little-endian PCM. Authored chime fragments,
/// with smooth attack/release and headroom; no random state or external assets.
#[cfg(any(windows, test))]
fn synthesize(cue: AudioCue, volume: f32) -> Vec<u8> {
    let (milliseconds, notes): (u32, &[(f64, f64, f64)]) = match cue {
        // Frequency, start (seconds), duration (seconds).
        AudioCue::Arrival => (200, &[(660.0, 0.0, 0.13), (880.0, 0.07, 0.13)]),
        AudioCue::Reward => (220, &[(784.0, 0.0, 0.14), (1176.0, 0.06, 0.16)]),
        AudioCue::Equip => (90, &[(523.25, 0.0, 0.09), (1046.5, 0.0, 0.06)]),
    };
    let samples = SAMPLE_RATE * milliseconds / 1_000;
    let data_bytes = samples * 2;
    let mut wave = Vec::with_capacity(44 + data_bytes as usize);
    wave.extend_from_slice(b"RIFF");
    wave.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    wave.extend_from_slice(b"WAVEfmt ");
    wave.extend_from_slice(&16_u32.to_le_bytes());
    wave.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    wave.extend_from_slice(&1_u16.to_le_bytes()); // mono
    wave.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wave.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wave.extend_from_slice(&2_u16.to_le_bytes()); // block alignment
    wave.extend_from_slice(&16_u16.to_le_bytes());
    wave.extend_from_slice(b"data");
    wave.extend_from_slice(&data_bytes.to_le_bytes());
    let gain = f64::from(normalize_volume(volume)) * 0.18 * f64::from(i16::MAX);
    for index in 0..samples {
        let time = f64::from(index) / f64::from(SAMPLE_RATE);
        let mut value = 0.0;
        for &(frequency, start, duration) in notes {
            let local = time - start;
            if (0.0..duration).contains(&local) {
                let position = local / duration;
                let envelope =
                    (std::f64::consts::PI * position).sin().powi(2) * (-3.0 * position).exp();
                let phase = std::f64::consts::TAU * frequency * local;
                value += envelope * (0.85 * phase.sin() + 0.15 * (2.0 * phase).sin());
            }
        }
        // Explicit silent endpoints avoid clicks at buffer boundaries.
        let sample = if index == 0 || index + 1 == samples {
            0
        } else {
            (value * gain / notes.len() as f64).round() as i16
        };
        wave.extend_from_slice(&sample.to_le_bytes());
    }
    wave
}

#[cfg(windows)]
fn play_wave(wave: &[u8]) {
    use std::ffi::c_void;

    #[link(name = "winmm")]
    unsafe extern "system" {
        fn PlaySoundW(sound: *const u16, module: *mut c_void, flags: u32) -> i32;
    }
    const SND_MEMORY: u32 = 0x0004;
    const SND_NODEFAULT: u32 = 0x0002;
    const SND_NOSTOP: u32 = 0x0010;
    // SND_SYNC is zero. Do not add SND_ASYNC or SND_SYSTEM.
    // DWORD-aligned storage also satisfies the wide-pointer ABI. Padding is
    // outside the RIFF length. Windows targets are little-endian.
    let aligned: Vec<u32> = wave
        .chunks(4)
        .map(|chunk| {
            let mut word = [0_u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            u32::from_le_bytes(word)
        })
        .collect();
    // SAFETY: aligned contains a complete internally generated PCM RIFF file
    // and remains owned here until synchronous playback returns. SND_MEMORY
    // interprets the pointer as bytes, not a NUL-terminated UTF-16 filename.
    // No resource module is used. Failure is silent; no system fallback sound.
    unsafe {
        PlaySoundW(
            aligned.as_ptr().cast(),
            std::ptr::null_mut(),
            SND_MEMORY | SND_NODEFAULT | SND_NOSTOP,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    fn samples(bytes: &[u8]) -> Vec<i16> {
        bytes[44..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect()
    }

    #[test]
    fn pcm_headers_duration_and_headroom_are_valid_for_every_cue() {
        for cue in [AudioCue::Arrival, AudioCue::Reward, AudioCue::Equip] {
            let wave = synthesize(cue, 1.0);
            assert_eq!(&wave[..4], b"RIFF");
            assert_eq!(&wave[8..16], b"WAVEfmt ");
            assert_eq!(u32_at(&wave, 4) as usize + 8, wave.len());
            assert_eq!(u32_at(&wave, 16), 16);
            assert_eq!(&wave[20..24], &[1, 0, 1, 0]);
            assert_eq!(u32_at(&wave, 24), SAMPLE_RATE);
            assert_eq!(u32_at(&wave, 28), SAMPLE_RATE * 2);
            assert_eq!(&wave[32..36], &[2, 0, 16, 0]);
            assert_eq!(&wave[36..40], b"data");
            assert_eq!(u32_at(&wave, 40) as usize + 44, wave.len());
            let pcm = samples(&wave);
            assert!(!pcm.is_empty() && pcm.len() <= SAMPLE_RATE as usize * 3 / 10);
            assert_eq!(pcm.first(), Some(&0));
            assert_eq!(pcm.last(), Some(&0));
            assert!(pcm.iter().any(|&sample| sample != 0));
            assert!(pcm.iter().all(|&sample| i32::from(sample).abs() < 6_000));
        }
    }

    #[test]
    fn cues_are_original_distinct_and_repeatable() {
        let arrival = synthesize(AudioCue::Arrival, 0.5);
        assert_eq!(arrival, synthesize(AudioCue::Arrival, 0.5));
        assert_ne!(arrival, synthesize(AudioCue::Reward, 0.5));
        assert_ne!(arrival, synthesize(AudioCue::Equip, 0.5));
    }

    #[test]
    fn volume_scales_pcm_without_touching_header_or_clipping() {
        let full = synthesize(AudioCue::Reward, 1.0);
        let half = synthesize(AudioCue::Reward, 0.5);
        assert_eq!(&full[..44], &half[..44]);
        for (a, b) in samples(&full).iter().zip(samples(&half)) {
            assert!((i32::from(*a) - 2 * i32::from(b)).abs() <= 1);
        }
        assert_eq!(full, synthesize(AudioCue::Reward, 5.0));
        for volume in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(normalize_volume(volume), 0.0);
            assert!(
                samples(&synthesize(AudioCue::Reward, volume))
                    .iter()
                    .all(|&s| s == 0)
            );
        }
    }

    #[test]
    fn construction_and_silent_calls_never_start_a_worker() {
        let mut player = AudioPlayer::new();
        for volume in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            player.play(AudioCue::Arrival, volume);
        }
        #[cfg(windows)]
        {
            assert!(player.worker.is_none());
            assert!(!player.attempted);
        }
    }

    #[test]
    fn queue_retains_only_one_waiting_request() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker = Worker {
            sender,
            stopped: Arc::new(AtomicBool::new(false)),
        };
        for _ in 0..10_000 {
            worker.enqueue(AudioCue::Equip, 0.5);
        }
        assert_eq!(receiver.try_recv().unwrap().cue, AudioCue::Equip);
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }

    #[test]
    fn expired_and_silent_requests_do_not_reach_playback() {
        let (sender, receiver) = mpsc::sync_channel(2);
        sender
            .send(Request {
                cue: AudioCue::Arrival,
                volume: 0.5,
                created: Instant::now() - Duration::from_secs(1),
            })
            .unwrap();
        sender
            .send(Request {
                cue: AudioCue::Reward,
                volume: 0.0,
                created: Instant::now(),
            })
            .unwrap();
        drop(sender);
        run_worker(receiver, Arc::new(AtomicBool::new(false)), |_| {
            panic!("unexpected sound")
        });
    }

    #[test]
    fn shutdown_does_not_join_playback_and_discards_waiting_cue() {
        struct Finished(mpsc::Sender<()>);
        impl Drop for Finished {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        let finished = Finished(finished_tx);
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = Arc::clone(&calls);
        let worker = Worker::spawn(move |_| {
            let _keep_until_worker_exit = &finished;
            count.fetch_add(1, Ordering::SeqCst);
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap(); // Simulated device, no actual sound.
        })
        .unwrap();
        worker.enqueue(AudioCue::Arrival, 0.5);
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        worker.enqueue(AudioCue::Reward, 0.5);
        let (dropped_tx, dropped_rx) = mpsc::channel();
        std::thread::spawn(move || {
            drop(worker);
            dropped_tx.send(()).unwrap();
        });
        let dropped = dropped_rx.recv_timeout(Duration::from_secs(2));
        release_tx.send(()).unwrap(); // Release even if a regression joined the worker.
        assert!(dropped.is_ok(), "shutdown waited for playback");
        finished_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
