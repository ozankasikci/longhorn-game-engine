# iOS Export System Design

**Date:** 2025-12-11
**Status:** Design Approved

## Overview

This document describes the design for the iOS export feature in the Longhorn game engine. The system will generate standalone Xcode projects from Longhorn game projects, similar to Unity's iOS export functionality.

## Design Decisions

The following decisions were made during the design process:

1. **Asset Bundling:** Bundle all game assets (textures, scripts, game.json) into the Xcode project for a completely standalone build
2. **Export Triggers:** Support both editor UI and CLI tool for maximum flexibility
3. **Configuration Level:** Essential mobile settings (app name, bundle ID, version, display name, icon)
4. **Rust Compilation:** Pre-compiled static libraries for fast exports without requiring Rust toolchain
5. **iOS Deployment Target:** iOS 13.0 for widest device compatibility

## Architecture

### Key Components

1. **Export Engine** (`longhorn-export` crate): Core logic for generating Xcode projects
2. **Template System**: Pre-configured Xcode project template with Swift/Objective-C bridge code
3. **Asset Bundler**: Copies and organizes game assets into the Xcode project
4. **Static Library Builder**: Pre-compiled Rust libraries for iOS (arm64) and simulator (x86_64, arm64)
5. **Configuration Manager**: Handles export settings

### Export Flow

1. User triggers export (via UI or CLI) and provides configuration
2. Export engine validates game project and configuration
3. Template Xcode project is copied to output directory
4. Game assets are bundled into the Xcode project's Resources
5. Pre-compiled static libraries are copied into the project
6. Info.plist and project settings are configured based on user input
7. App icon is processed and added to Assets.xcassets
8. Export completes with ready-to-build Xcode project

The generated Xcode project will contain a minimal Swift bridge layer that initializes the Longhorn engine (via FFI) and passes iOS lifecycle events to the Rust code.

## Exported Xcode Project Structure

```
MyGame/                          # Output directory
├── MyGame.xcodeproj/           # Xcode project
├── MyGame/                      # App source directory
│   ├── AppDelegate.swift       # iOS app lifecycle
│   ├── GameViewController.swift # Main game view controller
│   ├── LonghornBridge.swift    # Swift → Rust FFI bridge
│   ├── Info.plist              # App metadata (generated)
│   ├── Assets.xcassets/        # App icon and launch images
│   │   └── AppIcon.appiconset/ # Generated from provided icon
│   ├── LaunchScreen.storyboard # Default launch screen
│   └── GameResources/          # Bundled game assets
│       ├── game.json           # Game manifest
│       ├── scripts/            # TypeScript game scripts
│       │   └── main.ts
│       └── assets/             # Textures, sounds, etc.
│           └── textures/
└── Frameworks/                  # Static libraries
    ├── liblonghorn_mobile.a    # iOS device (arm64)
    └── liblonghorn_mobile_sim.a # Simulator (x86_64 + arm64)
```

### Key Design Details

- **GameResources/** is added as a bundle resource in Xcode, making all game files accessible via `Bundle.main` at runtime
- Static libraries are linked as frameworks with proper architecture slicing (device vs simulator)
- The Swift bridge layer is minimal (~200 lines) and handles: window setup, Metal rendering surface, touch event forwarding, and app lifecycle events
- All game-specific logic stays in Rust; Swift is just the platform adapter

## Export Configuration

The export system uses a dedicated `ios-export.json` file in the game project root to store iOS-specific settings.

### Configuration File Format

```json
{
  "app_name": "MyAwesomeGame",
  "display_name": "My Awesome Game",
  "bundle_id": "com.mystudio.myawesomegame",
  "version": "1.0.0",
  "build_number": "1",
  "app_icon": "assets/app-icon.png"
}
```

### Configuration Fields

- **app_name**: Internal Xcode project/scheme name (alphanumeric + underscores only)
- **display_name**: User-facing name shown under the app icon (max 30 chars)
- **bundle_id**: Reverse domain identifier (e.g., com.company.gamename)
- **version**: Marketing version string (e.g., "1.0.0")
- **build_number**: Incremental build number for App Store submissions
- **app_icon**: Path to 1024×1024 PNG icon (will be auto-resized to all required sizes)

### Defaults and Generation

If `ios-export.json` doesn't exist, the export tool will generate sensible defaults from `game.json`:
- app_name: Derived from game.json's "name" field (sanitized)
- display_name: Same as game name (truncated if > 30 chars)
- bundle_id: "com.longhorn.{sanitized-game-name}"
- version: "1.0.0"
- app_icon: Uses a default Longhorn logo if not specified

The CLI and editor UI both allow overriding these values at export time without modifying the config file.

## CLI Tool

The CLI tool will be a new binary `longhorn-export` with a focused interface.

### Command Structure

```bash
# Basic export with defaults
longhorn-export ios /path/to/game-project

# Specify output directory
longhorn-export ios /path/to/game-project --output ./builds/ios

# Override configuration at export time
longhorn-export ios /path/to/game-project \
  --app-name "MyGame" \
  --bundle-id "com.example.mygame" \
  --version "2.0.0"

# Use custom config file
longhorn-export ios /path/to/game-project --config custom-ios.json
```

### Available Options

- `--output, -o <path>`: Output directory (default: `./ios-export`)
- `--app-name <name>`: Override app name
- `--display-name <name>`: Override display name
- `--bundle-id <id>`: Override bundle identifier
- `--version <version>`: Override version string
- `--build-number <num>`: Override build number
- `--app-icon <path>`: Override app icon path
- `--config <path>`: Use custom config file (default: `ios-export.json`)
- `--force`: Overwrite existing output directory

### Error Handling

The CLI validates inputs and provides clear error messages for:
- Invalid game project (missing game.json)
- Invalid bundle ID format
- Missing or invalid app icon
- Output directory conflicts
- Missing static library dependencies

Exit codes: 0 (success), 1 (validation error), 2 (export failed)

## Editor UI Integration

### UI Location

New top-level menu item: **File → Export → iOS...**

### Export Dialog Workflow

Modal dialog with:

1. **Configuration Section** (editable form):
   - App Name (text field)
   - Display Name (text field)
   - Bundle ID (text field with validation indicator)
   - Version (text field, semantic version format)
   - Build Number (number input)
   - App Icon (file picker with preview thumbnail)

2. **Output Section**:
   - Output Directory (path picker, default: `./ios-export`)
   - "Open in Xcode after export" checkbox (default: checked)

3. **Action Buttons**:
   - "Save Configuration" (saves to `ios-export.json`)
   - "Export" (primary button)
   - "Cancel"

### Progress Feedback

During export, the dialog shows a progress bar with status messages:
- "Validating game project..."
- "Copying template project..."
- "Bundling game assets..."
- "Processing app icon..."
- "Configuring Xcode project..."
- "Export complete!"

If "Open in Xcode after export" is checked, the editor automatically runs `open MyGame.xcodeproj` on completion.

### Error Display

Validation errors appear inline next to the relevant field. Export errors show in a modal with actionable error messages.

## Build System & Static Libraries

### Static Library Generation

A new build script (`scripts/build-ios-libs.sh`) compiles the Rust code for iOS:

```bash
# Build for iOS devices (arm64)
cargo build --release --target aarch64-apple-ios \
  --package longhorn-mobile

# Build for iOS simulator (x86_64 + arm64)
cargo build --release --target x86_64-apple-ios \
  --package longhorn-mobile
cargo build --release --target aarch64-apple-ios-sim \
  --package longhorn-mobile

# Combine simulator architectures into universal binary
lipo -create \
  target/x86_64-apple-ios/release/liblonghorn_mobile.a \
  target/aarch64-apple-ios-sim/release/liblonghorn_mobile.a \
  -output liblonghorn_mobile_sim.a
```

### Library Placement

The export tool copies pre-built libraries from `target/` to the Xcode project's `Frameworks/` directory. The Xcode project automatically selects the correct library based on build destination.

### Xcode Project Configuration

- **Library Search Paths**: `$(PROJECT_DIR)/Frameworks`
- **Other Linker Flags**: `-llonghorn_mobile` (device) or `-llonghorn_mobile_sim` (simulator)
- **Architecture Conditional Linking**: Uses build settings to link the correct library
- **Deployment Target**: iOS 13.0
- **Supported Architectures**: arm64 (device), x86_64 + arm64 (simulator)

### FFI Bridge Header

A C header file (`longhorn_mobile.h`) exposes the Rust FFI functions to Swift:
- `longhorn_init()`
- `longhorn_load_game()`
- `longhorn_update()`
- `longhorn_handle_touch()`
- etc.

## Testing & Deployment

### Testing the Exported Project

After export, developers can immediately test:

1. **Simulator Testing**: Open `.xcodeproj`, select iOS Simulator, and press Run (⌘R)
2. **Device Testing**: Connect iOS device, select it, and Run
   - Requires signing: Set Team in project settings (automatic signing recommended)
3. **Build Verification**: Product → Build (⌘B) to verify compilation

### App Store Preparation

For App Store submission:
- Configure signing with Distribution certificate
- Increment build number for each submission
- Archive the app (Product → Archive)
- Upload via Xcode Organizer or `xcrun altool`

### Common Issues & Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| "Library not found" | Missing static lib | Re-run export or rebuild iOS libs |
| "Code signing failed" | No team/certificate | Set Team in Signing & Capabilities |
| "Game resources not found" | Bundle path issue | Check GameResources is in Copy Bundle Resources |
| Simulator crash on startup | Architecture mismatch | Ensure simulator lib includes x86_64 + arm64 |

### Developer Documentation

The export tool generates a `README.md` in the exported project with:
- How to open and build the project
- How to change bundle ID/signing
- How to update game assets (re-export vs manual copy)
- Troubleshooting common issues

## Implementation Considerations

### New Components to Create

1. **longhorn-export crate**: Core export logic
2. **Xcode template project**: Pre-configured template with Swift bridge
3. **Build script**: iOS library compilation automation
4. **FFI layer**: Rust → C → Swift bridge for iOS platform integration
5. **Editor UI**: Export dialog and menu integration
6. **CLI binary**: Command-line export tool

### Dependencies

- `plist` crate for generating Info.plist files
- `image` crate for app icon processing and resizing
- `walkdir` for asset bundling
- Template files embedded in binary using `include_str!` or similar

## Success Criteria

The iOS export feature will be considered successful when:

1. A game can be exported to a working Xcode project in under 10 seconds
2. The exported project builds and runs on iOS Simulator without errors
3. The exported project can be built for device with only signing configuration
4. Game assets load correctly from the bundle at runtime
5. Touch input works correctly
6. App lifecycle events (suspend/resume) are handled properly
7. The export process provides clear error messages for common issues
