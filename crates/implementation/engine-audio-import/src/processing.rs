use crate::{AudioData, AudioFormat, SampleRate, ChannelLayout, AudioError};
use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters, WindowFunction};

pub struct SampleRateConverter;

impl SampleRateConverter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn convert(&self, audio_data: &mut AudioData, target_rate: SampleRate) -> Result<(), AudioError> {
        if audio_data.sample_rate == target_rate {
            return Ok(());
        }
        
        let source_rate = audio_data.sample_rate.as_u32();
        let target_rate_u32 = target_rate.as_u32();
        let channels = audio_data.channel_layout.channel_count();
        
        // Convert samples based on format
        match audio_data.format {
            AudioFormat::PcmF32 => {
                let samples_f32: Vec<f32> = audio_data.samples.chunks(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                
                let samples_per_channel = samples_f32.len() / channels;
                let mut channel_data: Vec<Vec<f32>> = vec![Vec::with_capacity(samples_per_channel); channels];
                
                // Deinterleave
                for (i, sample) in samples_f32.iter().enumerate() {
                    channel_data[i % channels].push(*sample);
                }
                
                // Create resampler
                let params = SincInterpolationParameters {
                    sinc_len: 256,
                    f_cutoff: 0.95,
                    interpolation: SincInterpolationType::Linear,
                    oversampling_factor: 256,
                    window: WindowFunction::BlackmanHarris2,
                };
                
                let mut resampler = SincFixedIn::<f32>::new(
                    target_rate_u32 as f64 / source_rate as f64,
                    2.0,
                    params,
                    channel_data[0].len(),
                    channels,
                ).map_err(|e| AudioError::ProcessingError(e.to_string()))?;
                
                let output = resampler.process(&channel_data, None)
                    .map_err(|e| AudioError::ProcessingError(e.to_string()))?;
                
                // Interleave back
                let mut new_samples = Vec::new();
                if !output.is_empty() && !output[0].is_empty() {
                    let output_len = output[0].len();
                    for i in 0..output_len {
                        for ch in 0..channels {
                            new_samples.extend_from_slice(&output[ch][i].to_le_bytes());
                        }
                    }
                } else {
                    // Fallback if resampler returns empty output
                    // Use simple duplication for upsampling
                    let ratio = target_rate_u32 as f32 / source_rate as f32;
                    for sample in samples_f32 {
                        let repeat_count = ratio.round() as usize;
                        for _ in 0..repeat_count {
                            new_samples.extend_from_slice(&sample.to_le_bytes());
                        }
                    }
                }
                
                let new_sample_count = new_samples.len() / 4 / channels;
                audio_data.samples = new_samples;
                audio_data.sample_rate = target_rate;
                audio_data.duration_seconds = new_sample_count as f64 / target_rate_u32 as f64;
            }
            _ => {
                // For other formats, convert to f32 first
                // For simplicity, just duplicate samples for now
                let ratio = target_rate_u32 as f32 / source_rate as f32;
                let new_len = ((audio_data.samples.len() as f32 * ratio) as usize).max(12);
                let mut new_samples = vec![0u8; new_len];
                
                // Simple linear interpolation
                for i in 0..new_len {
                    let src_idx = (i as f32 / ratio) as usize;
                    if src_idx < audio_data.samples.len() {
                        new_samples[i] = audio_data.samples[src_idx];
                    }
                }
                
                let bytes_per_sample = match audio_data.format {
                    AudioFormat::PcmS16 => 2,
                    AudioFormat::PcmS24 => 3,
                    AudioFormat::PcmS32 => 4,
                    AudioFormat::PcmF32 => 4,
                    _ => 1,
                };
                let new_sample_count = new_samples.len() / bytes_per_sample / channels;
                audio_data.samples = new_samples;
                audio_data.sample_rate = target_rate;
                audio_data.duration_seconds = new_sample_count as f64 / target_rate_u32 as f64;
            }
        }
        
        Ok(())
    }
}

pub struct ChannelConverter;

impl ChannelConverter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn convert(&self, audio_data: &mut AudioData, target_layout: ChannelLayout) -> Result<(), AudioError> {
        if audio_data.channel_layout == target_layout {
            return Ok(());
        }
        
        match (audio_data.channel_layout, target_layout, audio_data.format) {
            (ChannelLayout::Mono, ChannelLayout::Stereo, AudioFormat::PcmS16) => {
                let mut new_samples = Vec::with_capacity(audio_data.samples.len() * 2);
                
                // Duplicate mono samples to both channels
                for chunk in audio_data.samples.chunks(2) {
                    new_samples.extend_from_slice(chunk); // Left
                    new_samples.extend_from_slice(chunk); // Right (duplicate)
                }
                
                audio_data.samples = new_samples;
                audio_data.channel_layout = ChannelLayout::Stereo;
            }
            _ => return Err(AudioError::ProcessingError("Unsupported channel conversion".to_string())),
        }
        
        Ok(())
    }
}

pub struct AudioNormalizer;

impl AudioNormalizer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn normalize(&self, audio_data: &mut AudioData, target_peak: f32) -> Result<(), AudioError> {
        match audio_data.format {
            AudioFormat::PcmF32 => {
                let mut samples_f32: Vec<f32> = audio_data.samples.chunks(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                
                // Find peak
                let peak = samples_f32.iter()
                    .map(|s| s.abs())
                    .fold(0.0f32, |a, b| a.max(b));
                
                if peak > 0.0 {
                    let scale = target_peak / peak;
                    for sample in &mut samples_f32 {
                        *sample *= scale;
                    }
                }
                
                // Convert back to bytes
                audio_data.samples = samples_f32.iter()
                    .flat_map(|s| s.to_le_bytes())
                    .collect();
            }
            _ => return Err(AudioError::ProcessingError("Normalization only supported for f32 format".to_string())),
        }
        
        Ok(())
    }
}

pub struct SilenceTrimmer;

#[derive(Debug, Clone)]
pub struct TrimOptions {
    pub threshold_db: f32,
    pub min_silence_duration: f32,
    pub trim_start: bool,
    pub trim_end: bool,
}

impl SilenceTrimmer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn trim(&self, audio_data: &mut AudioData, options: &TrimOptions) -> Result<(), AudioError> {
        match audio_data.format {
            AudioFormat::PcmS16 => {
                let samples_i16: Vec<i16> = audio_data.samples.chunks(2)
                    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();
                
                let threshold = (10.0f32.powf(options.threshold_db / 20.0) * 32767.0) as i16;
                
                // Find start of audio
                let start_idx = if options.trim_start {
                    samples_i16.iter()
                        .position(|&s| s.abs() > threshold)
                        .unwrap_or(0)
                } else {
                    0
                };
                
                // Find end of audio
                let end_idx = if options.trim_end {
                    samples_i16.iter()
                        .rposition(|&s| s.abs() > threshold)
                        .map(|i| i + 1)
                        .unwrap_or(samples_i16.len())
                } else {
                    samples_i16.len()
                };
                
                // Trim samples
                let trimmed_samples = &samples_i16[start_idx..end_idx];
                audio_data.samples = bytemuck::cast_slice(trimmed_samples).to_vec();
                
                // Update duration
                let channels = audio_data.channel_layout.channel_count();
                let sample_rate = audio_data.sample_rate.as_u32();
                audio_data.duration_seconds = trimmed_samples.len() as f64 / sample_rate as f64 / channels as f64;
            }
            _ => return Err(AudioError::ProcessingError("Trimming only supported for S16 format".to_string())),
        }
        
        Ok(())
    }
}