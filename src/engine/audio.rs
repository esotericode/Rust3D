//! Small procedural sound mixer. CPAL only supplies the native output device.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Copy)]
pub enum Cue {
    Step,
    Jump,
    Land,
    Kick,
    Dive,
    Roll,
    Beacon,
}
impl Cue {
    fn duration(self) -> f32 {
        match self {
            Self::Step => 0.065,
            Self::Jump => 0.15,
            Self::Land => 0.12,
            Self::Kick => 0.16,
            Self::Dive => 0.18,
            Self::Roll => 0.16,
            Self::Beacon => 0.3,
        }
    }
    /// Relative pitch spread, so repeated footsteps and landings do not sound
    /// mechanically identical. The beacon chime keeps its exact notes.
    fn pitch_spread(self) -> f32 {
        match self {
            Self::Step => 0.14,
            Self::Land => 0.10,
            Self::Beacon => 0.,
            _ => 0.05,
        }
    }
}
struct Voice {
    cue: Cue,
    time: f32,
    volume: f32,
    noise: u32,
    pitch: f32,
    /// Oscillator phase in cycles. Accumulating it keeps sweeps monotonic and
    /// note changes click-free; `sin(t * f(t))` would bend sweeps through 0 Hz.
    phase: f32,
}
impl Voice {
    fn sample(&mut self, dt: f32) -> f32 {
        self.noise = self.noise.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (self.noise >> 8) as f32 / 8388607.5 - 1.;
        let t = self.time;
        let progress = (t / self.cue.duration()).clamp(0., 1.);
        let (frequency, grain) = match self.cue {
            Cue::Step => (110., 0.8),
            Cue::Jump => (300. + progress * 550., 0.05),
            Cue::Land => (95., 0.75),
            Cue::Kick => (650. - progress * 450., 0.35),
            Cue::Dive => (500. - progress * 350., 0.4),
            Cue::Roll => (180. + progress * 180., 0.25),
            Cue::Beacon => (
                if progress < 0.33 {
                    660.
                } else if progress < 0.66 {
                    880.
                } else {
                    1100.
                },
                0.,
            ),
        };
        let tone = (self.phase * std::f32::consts::TAU).sin();
        self.phase = (self.phase + frequency * self.pitch * dt).fract();
        let envelope = (t / 0.005).min(1.) * (1. - progress).powi(2);
        self.time += dt;
        (tone * (1. - grain) + noise * grain) * envelope * self.volume * 0.3
    }
}
#[derive(Default)]
struct Mixer {
    voices: Vec<Voice>,
}
impl Mixer {
    fn sample(&mut self, dt: f32) -> f32 {
        let result = self
            .voices
            .iter_mut()
            .map(|v| v.sample(dt))
            .sum::<f32>()
            .clamp(-0.95, 0.95);
        self.voices.retain(|v| v.time < v.cue.duration());
        result
    }
}
pub struct Audio {
    stream: Option<cpal::Stream>,
    mixer: Arc<Mutex<Mixer>>,
    failed: Arc<AtomicBool>,
    seed: AtomicU32,
}
impl Audio {
    pub fn new(enabled: bool) -> Self {
        let mixer = Arc::new(Mutex::new(Mixer::default()));
        let failed = Arc::new(AtomicBool::new(false));
        let stream = if enabled {
            Self::open(mixer.clone(), failed.clone())
        } else {
            None
        };
        Self {
            stream,
            mixer,
            failed,
            seed: AtomicU32::new(0x2545_f491),
        }
    }
    fn open(mixer: Arc<Mutex<Mixer>>, failed: Arc<AtomicBool>) -> Option<cpal::Stream> {
        let device = cpal::default_host().default_output_device()?;
        let supported = device.default_output_config().ok()?;
        let config = supported.config();
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => Self::build::<f32>(&device, &config, mixer, failed),
            cpal::SampleFormat::I16 => Self::build::<i16>(&device, &config, mixer, failed),
            cpal::SampleFormat::U16 => Self::build::<u16>(&device, &config, mixer, failed),
            _ => None,
        }?;
        stream.play().ok()?;
        Some(stream)
    }
    fn build<T: cpal::SizedSample + cpal::FromSample<f32>>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mixer: Arc<Mutex<Mixer>>,
        failed: Arc<AtomicBool>,
    ) -> Option<cpal::Stream> {
        let channels = config.channels as usize;
        let dt = 1. / config.sample_rate.0 as f32;
        device
            .build_output_stream(
                config,
                move |data: &mut [T], _| {
                    let mut mixer = mixer.lock().ok();
                    for frame in data.chunks_mut(channels) {
                        let sample = mixer.as_mut().map_or(0., |m| m.sample(dt));
                        for out in frame {
                            *out = T::from_sample(sample);
                        }
                    }
                },
                move |_| {
                    failed.store(true, Ordering::Relaxed);
                },
                None,
            )
            .ok()
    }
    pub fn available(&self) -> bool {
        self.stream.is_some() && !self.failed.load(Ordering::Relaxed)
    }
    pub fn play(&self, cue: Cue, volume: u32, gain: f32) {
        if volume == 0 || !self.available() {
            return;
        }
        let mut seed = self
            .seed
            .fetch_add(0x9e37_79b9, Ordering::Relaxed)
            .wrapping_mul(0x85eb_ca6b);
        seed ^= seed >> 13;
        let unit = (seed >> 8) as f32 / 16_777_216.;
        if let Ok(mut mixer) = self.mixer.lock() {
            if mixer.voices.len() < 16 {
                mixer.voices.push(Voice {
                    cue,
                    time: 0.,
                    volume: volume.min(100) as f32 / 100. * gain.clamp(0., 1.),
                    noise: seed | 1,
                    pitch: 1. + (unit - 0.5) * cue.pitch_spread(),
                    phase: 0.,
                });
            }
        }
    }
    pub fn silence(&self) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.voices.clear();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn procedural_cues_are_finite_bounded_and_finish() {
        for cue in [
            Cue::Step,
            Cue::Jump,
            Cue::Land,
            Cue::Kick,
            Cue::Dive,
            Cue::Roll,
            Cue::Beacon,
        ] {
            let mut mixer = Mixer {
                voices: vec![Voice {
                    cue,
                    time: 0.,
                    volume: 1.,
                    noise: 1,
                    pitch: 1.,
                    phase: 0.,
                }],
            };
            let mut peak = 0_f32;
            for _ in 0..24000 {
                let value = mixer.sample(1. / 48000.);
                assert!(value.is_finite() && value.abs() <= 0.95);
                peak = peak.max(value.abs());
            }
            assert!(peak > 0.01 && mixer.voices.is_empty());
        }
    }
    #[test]
    fn sweeps_stay_between_their_end_frequencies() {
        for (cue, low, high) in [
            (Cue::Jump, 300., 850.),
            (Cue::Kick, 200., 650.),
            (Cue::Dive, 150., 500.),
            (Cue::Roll, 180., 360.),
        ] {
            let mut voice = Voice {
                cue,
                time: 0.,
                volume: 1.,
                noise: 1,
                pitch: 1.,
                phase: 0.,
            };
            let dt = 1. / 48000.;
            while voice.time < cue.duration() {
                let before = voice.phase;
                voice.sample(dt);
                let frequency = (voice.phase - before).rem_euclid(1.) / dt;
                assert!(
                    frequency > low - 1. && frequency < high + 1.,
                    "{frequency} Hz outside {low}..{high}"
                );
            }
        }
    }
}
