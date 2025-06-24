use async_trait::async_trait;
use engine_asset_import::{AssetImporter, ImportContext, ImportError as AssetImportError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

pub mod analysis;
pub mod compression;
pub mod processing;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Unsupported audio format")]
    UnsupportedFormat,

    #[error("Invalid audio data: {0}")]
    InvalidData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WAV decode error: {0}")]
    WavError(#[from] hound::Error),

    #[error("Processing error: {0}")]
    ProcessingError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    PcmS16,
    PcmS24,
    PcmS32,
    PcmF32,
    Mp3,
    Opus,
    Vorbis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleRate {
    Hz22050,
    Hz44100,
    Hz48000,
    Hz96000,
    Custom(u32),
}

impl From<u32> for SampleRate {
    fn from(hz: u32) -> Self {
        match hz {
            22050 => Self::Hz22050,
            44100 => Self::Hz44100,
            48000 => Self::Hz48000,
            96000 => Self::Hz96000,
            hz => Self::Custom(hz),
        }
    }
}

impl SampleRate {
    pub fn as_u32(&self) -> u32 {
        match self {
            Self::Hz22050 => 22050,
            Self::Hz44100 => 44100,
            Self::Hz48000 => 48000,
            Self::Hz96000 => 96000,
            Self::Custom(hz) => *hz,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelLayout {
    Mono,
    Stereo,
    Surround5_1,
    Surround7_1,
}

impl ChannelLayout {
    pub fn channel_count(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Surround5_1 => 6,
            Self::Surround7_1 => 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub target_sample_rate: Option<SampleRate>,
    pub target_channel_layout: Option<ChannelLayout>,
    pub normalize: bool,
    pub compression_quality: f32,
    pub trim_silence: bool,
    pub silence_threshold: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            target_sample_rate: None,
            target_channel_layout: None,
            normalize: false,
            compression_quality: 0.9,
            trim_silence: false,
            silence_threshold: -60.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    pub format: AudioFormat,
    pub sample_rate: SampleRate,
    pub channel_layout: ChannelLayout,
    pub samples: Vec<u8>,
    pub duration_seconds: f64,
}

pub struct AudioImporter;

impl Default for AudioImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn import(&self, data: &[u8], _context: &ImportContext) -> Result<AudioData, AudioError> {
        // Check MP3 signature first (needs only 2 bytes)
        if data.len() >= 2 && (data.starts_with(&[0xFF, 0xFB]) || data.starts_with(&[0xFF, 0xF3])) {
            return Err(AudioError::UnsupportedFormat);
        }

        // Check OGG signature (needs 4 bytes)
        if data.len() >= 4 && data.starts_with(b"OggS") {
            return Err(AudioError::UnsupportedFormat);
        }

        // Check WAV signature (needs 12 bytes)
        if data.len() >= 12 && data.starts_with(b"RIFF") && data[8..12] == *b"WAVE" {
            return self.import_wav(data);
        }

        // If we get here, it's either too short or unsupported
        if data.len() < 2 {
            return Err(AudioError::InvalidData("Data too short".to_string()));
        }

        Err(AudioError::UnsupportedFormat)
    }

    fn import_wav(&self, data: &[u8]) -> Result<AudioData, AudioError> {
        use std::io::Cursor;
        let cursor = Cursor::new(data);
        let reader = hound::WavReader::new(cursor)?;
        let spec = reader.spec();

        // Determine format
        let format = match (spec.sample_format, spec.bits_per_sample) {
            (hound::SampleFormat::Int, 16) => AudioFormat::PcmS16,
            (hound::SampleFormat::Int, 24) => AudioFormat::PcmS24,
            (hound::SampleFormat::Int, 32) => AudioFormat::PcmS32,
            (hound::SampleFormat::Float, 32) => AudioFormat::PcmF32,
            _ => return Err(AudioError::UnsupportedFormat),
        };

        // Determine channel layout
        let channel_layout = match spec.channels {
            1 => ChannelLayout::Mono,
            2 => ChannelLayout::Stereo,
            _ => return Err(AudioError::UnsupportedFormat),
        };

        // Read samples
        let samples = match format {
            AudioFormat::PcmS16 => {
                let samples: Result<Vec<i16>, _> = reader.into_samples().collect();
                let samples = samples?;
                bytemuck::cast_slice(&samples).to_vec()
            }
            _ => return Err(AudioError::UnsupportedFormat),
        };

        let sample_count = samples.len() / 2; // 2 bytes per i16
        let duration_seconds = sample_count as f64 / spec.sample_rate as f64 / spec.channels as f64;

        Ok(AudioData {
            format,
            sample_rate: spec.sample_rate.into(),
            channel_layout,
            samples,
            duration_seconds,
        })
    }
}

#[async_trait]
impl AssetImporter for AudioImporter {
    type Asset = AudioData;

    fn supported_extensions(&self) -> &[&str] {
        &["wav", "mp3", "ogg", "flac"]
    }

    async fn import(
        &self,
        path: &Path,
        context: &ImportContext,
    ) -> Result<Self::Asset, AssetImportError> {
        // Read file data
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| AssetImportError::IoError(e.to_string()))?;

        // Import audio
        self.import(&data, context)
            .map_err(|e| AssetImportError::ProcessingError(e.to_string()))
    }
}
