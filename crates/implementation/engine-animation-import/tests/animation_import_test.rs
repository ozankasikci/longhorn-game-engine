// Test-Driven Development for Phase 20.7: Animation Import
// 
// This test defines the expected behavior for animation import functionality

use engine_animation_import::{
    AnimationImporter, AnimationData, AnimationFormat, AnimationSettings,
    AnimationError, Keyframe, Channel, InterpolationType
};
use engine_asset_import::{ImportContext, ImportSettings};

#[test]
fn test_gltf_animation_import() {
    // Test 1: Verify glTF animation import
    let importer = AnimationImporter::new();
    let context = ImportContext::new(ImportSettings::default());
    
    // Simplified glTF with animation (binary would be much larger in reality)
    let gltf_data = br#"{
        "asset": {"version": "2.0"},
        "animations": [{
            "name": "Rotate",
            "channels": [{
                "sampler": 0,
                "target": {"node": 0, "path": "rotation"}
            }],
            "samplers": [{
                "input": 0,
                "output": 1,
                "interpolation": "LINEAR"
            }]
        }],
        "accessors": [
            {
                "bufferView": 0,
                "count": 2,
                "type": "SCALAR",
                "componentType": 5126,
                "min": [0.0],
                "max": [1.0]
            },
            {
                "bufferView": 1,
                "count": 2,
                "type": "VEC4",
                "componentType": 5126
            }
        ]
    }"#;
    
    let result = importer.import(gltf_data, &context);
    
    // For now, expect unsupported (would need full glTF parser)
    assert!(result.is_err());
    assert!(matches!(result.err(), Some(AnimationError::UnsupportedFormat)));
}

#[test]
fn test_fbx_animation_import() {
    // Test 2: Verify FBX animation import
    let importer = AnimationImporter::new();
    let context = ImportContext::new(ImportSettings::default());
    
    // FBX header (binary format)
    let fbx_data = vec![
        b'K', b'a', b'y', b'd', b'a', b'r', b'a', b' ',  // Magic
        b'F', b'B', b'X', b' ', b'B', b'i', b'n', b'a',  // "FBX Bina"
        b'r', b'y', b' ', b' ', 0x1A, 0x00,              // "ry  " + separator
    ];
    
    let result = importer.import(&fbx_data, &context);
    assert!(result.is_err());
    assert!(matches!(result.err(), Some(AnimationError::UnsupportedFormat)));
}

#[test]
fn test_animation_data_structure() {
    // Test 3: Verify animation data structure
    let mut animation = AnimationData {
        name: "TestAnimation".to_string(),
        duration_seconds: 2.0,
        channels: vec![],
        format: AnimationFormat::Custom,
        compressed_data: None,
    };
    
    // Add rotation channel
    let rotation_channel = Channel {
        target_node: "Bone01".to_string(),
        property: engine_animation_import::PropertyType::Rotation,
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: vec![0.0, 0.0, 0.0, 1.0], // Quaternion identity
                interpolation: InterpolationType::Linear,
            },
            Keyframe {
                time: 1.0,
                value: vec![0.0, 0.707, 0.0, 0.707], // 90 degree Y rotation
                interpolation: InterpolationType::Linear,
            },
        ],
    };
    
    animation.channels.push(rotation_channel);
    
    assert_eq!(animation.channels.len(), 1);
    assert_eq!(animation.channels[0].keyframes.len(), 2);
    assert_eq!(animation.duration_seconds, 2.0);
}

#[test]
fn test_animation_settings() {
    // Test 4: Verify animation import settings
    let mut settings = AnimationSettings::default();
    
    assert_eq!(settings.optimize_keyframes, true);
    assert_eq!(settings.compression_tolerance, 0.001);
    assert_eq!(settings.target_fps, None);
    assert_eq!(settings.import_bone_animations, true);
    assert_eq!(settings.import_blend_shapes, true);
    
    // Modify settings
    settings.target_fps = Some(30.0);
    settings.optimize_keyframes = false;
    settings.compression_tolerance = 0.01;
    
    assert_eq!(settings.target_fps, Some(30.0));
}

#[test]
fn test_keyframe_optimization() {
    // Test 5: Verify keyframe optimization
    use engine_animation_import::processing::KeyframeOptimizer;
    
    let optimizer = KeyframeOptimizer::new();
    
    // Create channel with redundant keyframes
    let mut channel = Channel {
        target_node: "TestNode".to_string(),
        property: engine_animation_import::PropertyType::Position,
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: vec![0.0, 0.0, 0.0],
                interpolation: InterpolationType::Linear,
            },
            Keyframe {
                time: 0.5,
                value: vec![1.0, 0.0, 0.0],
                interpolation: InterpolationType::Linear,
            },
            Keyframe {
                time: 1.0,
                value: vec![2.0, 0.0, 0.0],
                interpolation: InterpolationType::Linear,
            },
            Keyframe {
                time: 1.5,
                value: vec![3.0, 0.0, 0.0],
                interpolation: InterpolationType::Linear,
            },
        ],
    };
    
    let result = optimizer.optimize(&mut channel, 0.01);
    assert!(result.is_ok());
    
    // Should remove the middle keyframe as it's on the linear path
    assert_eq!(channel.keyframes.len(), 2);
    assert_eq!(channel.keyframes[0].time, 0.0);
    assert_eq!(channel.keyframes[1].time, 1.5);
}

#[test]
fn test_animation_retargeting() {
    // Test 6: Verify animation retargeting
    use engine_animation_import::retargeting::{AnimationRetargeter, BoneMapping};
    
    let retargeter = AnimationRetargeter::new();
    
    let bone_mappings = vec![
        BoneMapping {
            source_bone: "mixamorig:Hips".to_string(),
            target_bone: "Hips".to_string(),
        },
        BoneMapping {
            source_bone: "mixamorig:Spine".to_string(),
            target_bone: "Spine".to_string(),
        },
    ];
    
    let mut animation = AnimationData {
        name: "Walk".to_string(),
        duration_seconds: 1.0,
        channels: vec![
            Channel {
                target_node: "mixamorig:Hips".to_string(),
                property: engine_animation_import::PropertyType::Position,
                keyframes: vec![
                    Keyframe {
                        time: 0.0,
                        value: vec![0.0, 1.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                ],
            },
        ],
        format: AnimationFormat::Custom,
        compressed_data: None,
    };
    
    let result = retargeter.retarget(&mut animation, &bone_mappings);
    assert!(result.is_ok());
    
    // Check bone names were remapped
    assert_eq!(animation.channels[0].target_node, "Hips");
}

#[test]
fn test_fps_conversion() {
    // Test 7: Verify FPS conversion
    use engine_animation_import::processing::FpsConverter;
    
    let converter = FpsConverter::new();
    
    let mut animation = AnimationData {
        name: "TestAnim".to_string(),
        duration_seconds: 1.0,
        channels: vec![
            Channel {
                target_node: "Node".to_string(),
                property: engine_animation_import::PropertyType::Position,
                keyframes: vec![
                    Keyframe {
                        time: 0.0,
                        value: vec![0.0, 0.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                    Keyframe {
                        time: 0.5,
                        value: vec![1.0, 0.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                    Keyframe {
                        time: 1.0,
                        value: vec![2.0, 0.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                ],
            },
        ],
        format: AnimationFormat::Custom,
        compressed_data: None,
    };
    
    // Convert from 60fps to 30fps
    let result = converter.convert(&mut animation, 60.0, 30.0);
    assert!(result.is_ok());
    
    // Keyframe times should remain the same (time is in seconds)
    assert_eq!(animation.channels[0].keyframes[0].time, 0.0);
    assert_eq!(animation.channels[0].keyframes[1].time, 0.5);
    assert_eq!(animation.channels[0].keyframes[2].time, 1.0);
}

#[test]
fn test_blend_shape_animation() {
    // Test 8: Verify blend shape animation
    let mut animation = AnimationData {
        name: "FaceAnimation".to_string(),
        duration_seconds: 2.0,
        channels: vec![],
        format: AnimationFormat::Custom,
        compressed_data: None,
    };
    
    // Add blend shape channel
    let blend_shape_channel = Channel {
        target_node: "Face".to_string(),
        property: engine_animation_import::PropertyType::BlendShape("smile".to_string()),
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: vec![0.0], // Weight 0-1
                interpolation: InterpolationType::Cubic,
            },
            Keyframe {
                time: 1.0,
                value: vec![1.0], // Full smile
                interpolation: InterpolationType::Cubic,
            },
            Keyframe {
                time: 2.0,
                value: vec![0.0], // Back to neutral
                interpolation: InterpolationType::Cubic,
            },
        ],
    };
    
    animation.channels.push(blend_shape_channel);
    
    assert_eq!(animation.channels[0].keyframes.len(), 3);
    if let engine_animation_import::PropertyType::BlendShape(name) = &animation.channels[0].property {
        assert_eq!(name, "smile");
    } else {
        panic!("Expected blend shape property");
    }
}

#[test]
fn test_animation_compression() {
    // Test 9: Verify animation compression
    use engine_animation_import::compression::{AnimationCompressor, CompressionOptions};
    
    let compressor = AnimationCompressor::new();
    
    let animation = AnimationData {
        name: "LongAnimation".to_string(),
        duration_seconds: 10.0,
        channels: vec![
            Channel {
                target_node: "Bone1".to_string(),
                property: engine_animation_import::PropertyType::Rotation,
                keyframes: (0..100).map(|i| Keyframe {
                    time: i as f32 * 0.1,
                    value: vec![0.0, 0.0, 0.0, 1.0],
                    interpolation: InterpolationType::Linear,
                }).collect(),
            },
        ],
        format: AnimationFormat::Custom,
        compressed_data: None,
    };
    
    let options = CompressionOptions {
        error_threshold: 0.001,
        quantization_bits: 16,
    };
    
    let result = compressor.compress(&animation, &options);
    assert!(result.is_ok());
    
    let compressed = result.unwrap();
    assert_eq!(compressed.format, AnimationFormat::Compressed);
    // Compressed data should be smaller
    assert!(compressed.compressed_data.as_ref().unwrap().len() < 100 * 4 * 4); // Less than uncompressed size
}

#[test]
fn test_animation_validation() {
    // Test 10: Verify animation validation
    use engine_animation_import::validation::AnimationValidator;
    
    let validator = AnimationValidator::new();
    
    // Valid animation
    let valid_animation = AnimationData {
        name: "ValidAnim".to_string(),
        duration_seconds: 1.0,
        channels: vec![
            Channel {
                target_node: "Node".to_string(),
                property: engine_animation_import::PropertyType::Position,
                keyframes: vec![
                    Keyframe {
                        time: 0.0,
                        value: vec![0.0, 0.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                    Keyframe {
                        time: 1.0,
                        value: vec![1.0, 0.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                ],
            },
        ],
        format: AnimationFormat::Custom,
        compressed_data: None,
    };
    
    let result = validator.validate(&valid_animation);
    assert!(result.is_ok());
    
    // Invalid animation (keyframes out of order)
    let invalid_animation = AnimationData {
        name: "InvalidAnim".to_string(),
        duration_seconds: 1.0,
        channels: vec![
            Channel {
                target_node: "Node".to_string(),
                property: engine_animation_import::PropertyType::Position,
                keyframes: vec![
                    Keyframe {
                        time: 1.0,
                        value: vec![1.0, 0.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                    Keyframe {
                        time: 0.0,
                        value: vec![0.0, 0.0, 0.0],
                        interpolation: InterpolationType::Linear,
                    },
                ],
            },
        ],
        format: AnimationFormat::Custom,
        compressed_data: None,
    };
    
    let result = validator.validate(&invalid_animation);
    assert!(result.is_err());
}