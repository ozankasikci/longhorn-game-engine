// Test-Driven Development for Phase 20.6: Audio Import
//
// This test defines the expected behavior for audio import functionality

use engine_asset_import::{ImportContext, ImportSettings};
use engine_audio_import::{
    AudioData, AudioError, AudioFormat, AudioImporter, AudioSettings, ChannelLayout, SampleRate,
};

#[test]
fn test_wav_import() {
    // Test 1: Verify WAV audio import
    let importer = AudioImporter::new();
    let context = ImportContext::new(ImportSettings::default());

    // Create minimal WAV header (44 bytes) + 1 sample
    let wav_data = vec![
        // RIFF header
        b'R', b'I', b'F', b'F', // ChunkID
        46, 0, 0, 0, // ChunkSize (file size - 8)
        b'W', b'A', b'V', b'E', // Format
        // fmt subchunk
        b'f', b'm', b't', b' ', // Subchunk1ID
        16, 0, 0, 0, // Subchunk1Size (16 for PCM)
        1, 0, // AudioFormat (1 = PCM)
        1, 0, // NumChannels (1 = mono)
        68, 172, 0, 0, // SampleRate (44100)
        136, 88, 1, 0, // ByteRate (44100 * 1 * 2)
        2, 0, // BlockAlign (1 * 2)
        16, 0, // BitsPerSample (16)
        // data subchunk
        b'd', b'a', b't', b'a', // Subchunk2ID
        2, 0, 0, 0, // Subchunk2Size
        0, 0, // Sample data (silence)
    ];

    let result = importer.import(&wav_data, &context);
    assert!(result.is_ok());

    let audio_data = result.unwrap();
    assert_eq!(audio_data.format, AudioFormat::PcmS16);
    assert_eq!(audio_data.sample_rate, SampleRate::Hz44100);
    assert_eq!(audio_data.channel_layout, ChannelLayout::Mono);
    assert_eq!(audio_data.samples.len(), 2); // 1 sample * 2 bytes
}

#[test]
fn test_mp3_import() {
    // Test 2: Verify MP3 audio import
    let importer = AudioImporter::new();
    let context = ImportContext::new(ImportSettings::default());

    // MP3 header (simplified - would need real MP3 decoder)
    let mp3_data = vec![
        0xFF, 0xFB, // Frame sync + MPEG1 Layer3
        0x90, 0x00, // 128kbps, 44.1kHz, no padding, stereo
    ];

    let result = importer.import(&mp3_data, &context);
    // For now, expect unsupported format
    assert!(result.is_err());
    if let Err(e) = &result {
        println!("MP3 import error: {:?}", e);
    }
    assert!(matches!(result.err(), Some(AudioError::UnsupportedFormat)));
}

#[test]
fn test_ogg_vorbis_import() {
    // Test 3: Verify OGG Vorbis import
    let importer = AudioImporter::new();
    let context = ImportContext::new(ImportSettings::default());

    // OGG header
    let ogg_data = vec![
        b'O', b'g', b'g', b'S', // OGG magic
        0, 2, // Version, header type
    ];

    let result = importer.import(&ogg_data, &context);
    // For now, expect unsupported format
    assert!(result.is_err());
}

#[test]
fn test_audio_settings() {
    // Test 4: Verify audio import settings
    let mut settings = AudioSettings::default();

    assert_eq!(settings.target_sample_rate, None);
    assert_eq!(settings.target_channel_layout, None);
    assert!(!settings.normalize);
    assert_eq!(settings.compression_quality, 0.9);

    // Modify settings
    settings.target_sample_rate = Some(SampleRate::Hz48000);
    settings.target_channel_layout = Some(ChannelLayout::Stereo);
    settings.normalize = true;
    settings.trim_silence = true;
    settings.silence_threshold = -40.0;

    assert_eq!(settings.silence_threshold, -40.0);
}

#[test]
fn test_sample_rate_conversion() {
    // Test 5: Verify sample rate conversion
    use engine_audio_import::processing::SampleRateConverter;

    let converter = SampleRateConverter::new();

    // Create test audio at 22050Hz
    let mut audio_data = AudioData {
        format: AudioFormat::PcmF32,
        sample_rate: SampleRate::Hz22050,
        channel_layout: ChannelLayout::Mono,
        samples: vec![0, 0, 0, 0, 255, 255, 127, 127], // 2 f32 samples
        duration_seconds: 2.0 / 22050.0,
    };

    // Convert to 44100Hz
    let result = converter.convert(&mut audio_data, SampleRate::Hz44100);
    if let Err(e) = &result {
        println!("Sample rate conversion error: {:?}", e);
    }
    assert!(result.is_ok());

    println!(
        "Sample count after conversion: {}",
        audio_data.samples.len()
    );
    assert_eq!(audio_data.sample_rate, SampleRate::Hz44100);
    // Should have approximately double the samples
    assert!(audio_data.samples.len() >= 12); // At least 3 samples * 4 bytes
}

#[test]
fn test_channel_conversion() {
    // Test 6: Verify channel layout conversion
    use engine_audio_import::processing::ChannelConverter;

    let converter = ChannelConverter::new();

    // Create mono audio
    let mut audio_data = AudioData {
        format: AudioFormat::PcmS16,
        sample_rate: SampleRate::Hz44100,
        channel_layout: ChannelLayout::Mono,
        samples: vec![0, 1, 0, 2], // 2 samples
        duration_seconds: 2.0 / 44100.0,
    };

    // Convert to stereo
    let result = converter.convert(&mut audio_data, ChannelLayout::Stereo);
    assert!(result.is_ok());

    assert_eq!(audio_data.channel_layout, ChannelLayout::Stereo);
    assert_eq!(audio_data.samples.len(), 8); // 2 samples * 2 channels * 2 bytes
}

#[test]
fn test_audio_normalization() {
    // Test 7: Verify audio normalization
    use engine_audio_import::processing::AudioNormalizer;

    let normalizer = AudioNormalizer::new();

    // Create quiet audio
    let mut audio_data = AudioData {
        format: AudioFormat::PcmF32,
        sample_rate: SampleRate::Hz44100,
        channel_layout: ChannelLayout::Mono,
        samples: vec![
            0, 0, 0, 0, // 0.0
            0, 0, 0, 62, // 0.25
            0, 0, 0, 190, // -0.25
            0, 0, 128, 62, // 0.5
        ],
        duration_seconds: 4.0 / 44100.0,
    };

    let result = normalizer.normalize(&mut audio_data, 1.0);
    assert!(result.is_ok());

    // Audio should be louder after normalization
    let max_sample = audio_data
        .samples
        .chunks(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]).abs())
        .fold(0.0f32, |a, b| a.max(b));

    assert!(max_sample > 0.9); // Should be normalized close to 1.0
}

#[test]
fn test_silence_trimming() {
    // Test 8: Verify silence trimming
    use engine_audio_import::processing::SilenceTrimmer;

    let trimmer = SilenceTrimmer::new();

    // Create audio with silence at start and end
    let mut audio_data = AudioData {
        format: AudioFormat::PcmS16,
        sample_rate: SampleRate::Hz44100,
        channel_layout: ChannelLayout::Mono,
        samples: vec![
            0, 0, // Silence
            0, 0, // Silence
            100, 0, // Sound
            100, 0, // Sound
            0, 0, // Silence
            0, 0, // Silence
        ],
        duration_seconds: 6.0 / 44100.0,
    };

    let options = engine_audio_import::processing::TrimOptions {
        threshold_db: -60.0,
        min_silence_duration: 0.0,
        trim_start: true,
        trim_end: true,
    };

    let result = trimmer.trim(&mut audio_data, &options);
    assert!(result.is_ok());

    // Should have removed silence
    assert_eq!(audio_data.samples.len(), 4); // Only the sound samples
}

#[test]
fn test_audio_compression() {
    // Test 9: Verify audio compression
    use engine_audio_import::compression::{AudioCompressor, CompressionFormat};

    let compressor = AudioCompressor::new();

    let audio_data = AudioData {
        format: AudioFormat::PcmS16,
        sample_rate: SampleRate::Hz44100,
        channel_layout: ChannelLayout::Stereo,
        samples: vec![0; 44100 * 2 * 2], // 1 second of stereo silence
        duration_seconds: 1.0,
    };

    let options = engine_audio_import::compression::CompressionOptions {
        format: CompressionFormat::Opus,
        quality: 0.8,
        bitrate: Some(128000),
    };

    let result = compressor.compress(&audio_data, &options);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.format, AudioFormat::Opus);
    // Compressed size should be much smaller
    assert!(compressed.samples.len() < audio_data.samples.len() / 4);
}

#[test]
fn test_audio_analysis() {
    // Test 10: Verify audio analysis
    use engine_audio_import::analysis::AudioAnalyzer;

    let analyzer = AudioAnalyzer::new();

    // Create test signal
    let mut samples = Vec::new();
    for i in 0..44100 {
        let value = (i as f32 * 2.0 * std::f32::consts::PI * 440.0 / 44100.0).sin();
        samples.extend_from_slice(&value.to_le_bytes());
    }

    let audio_data = AudioData {
        format: AudioFormat::PcmF32,
        sample_rate: SampleRate::Hz44100,
        channel_layout: ChannelLayout::Mono,
        samples,
        duration_seconds: 1.0,
    };

    let analysis = analyzer.analyze(&audio_data).unwrap();

    assert!(analysis.peak_amplitude > 0.9);
    assert!(analysis.peak_amplitude <= 1.0);
    assert!(analysis.rms_level > 0.0);
    assert!(analysis.rms_level < analysis.peak_amplitude);
    assert!(!analysis.is_clipping);
    assert_eq!(analysis.duration_seconds, 1.0);

    // Should detect the 440Hz frequency
    assert!(analysis.dominant_frequency > 430.0 && analysis.dominant_frequency < 450.0);
}
