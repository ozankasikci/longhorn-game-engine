use crate::{AudioData, AudioError, AudioFormat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAnalysis {
    pub peak_amplitude: f32,
    pub rms_level: f32,
    pub is_clipping: bool,
    pub duration_seconds: f64,
    pub dominant_frequency: f32,
}

pub struct AudioAnalyzer;

impl AudioAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, audio_data: &AudioData) -> Result<AudioAnalysis, AudioError> {
        match audio_data.format {
            AudioFormat::PcmF32 => {
                let samples_f32: Vec<f32> = audio_data
                    .samples
                    .chunks(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                // Calculate peak amplitude
                let peak_amplitude = samples_f32
                    .iter()
                    .map(|s| s.abs())
                    .fold(0.0f32, |a, b| a.max(b));

                // Calculate RMS
                let sum_squares: f32 = samples_f32.iter().map(|s| s * s).sum();
                let rms_level = (sum_squares / samples_f32.len() as f32).sqrt();

                // Check for clipping
                let is_clipping = samples_f32.iter().any(|&s| s.abs() >= 1.0);

                // Simple frequency detection using zero crossings
                let mut zero_crossings = 0;
                let mut last_sign = samples_f32[0] >= 0.0;

                for &sample in &samples_f32[1..] {
                    let current_sign = sample >= 0.0;
                    if current_sign != last_sign {
                        zero_crossings += 1;
                    }
                    last_sign = current_sign;
                }

                // Estimate frequency from zero crossings
                let dominant_frequency =
                    (zero_crossings as f32 / 2.0) / audio_data.duration_seconds as f32;

                Ok(AudioAnalysis {
                    peak_amplitude,
                    rms_level,
                    is_clipping,
                    duration_seconds: audio_data.duration_seconds,
                    dominant_frequency,
                })
            }
            _ => Err(AudioError::ProcessingError(
                "Analysis only supported for f32 format".to_string(),
            )),
        }
    }
}
