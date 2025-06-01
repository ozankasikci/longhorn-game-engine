# Engine UI System

A comprehensive, Unity-style unified component system for the mobile game engine editor. This crate provides professional, cohesive UI components with consistent theming, typography, and styling.

## 🎨 Features

### Theme Engine
- **Dual Theme Support**: Unity Dark and Unity Light themes
- **Dynamic Theme Switching**: Runtime theme changes with hot-reload
- **Custom Theme Creation**: Build your own themes with complete control
- **CSS Generation**: Automatic CSS generation for all components
- **Platform Detection**: Auto-detects system preferences (macOS/Windows/Linux)

### Color System
- **Professional Palettes**: Unity-inspired color schemes
- **Accessibility**: WCAG AA compliant contrast ratios
- **Color Utilities**: Interpolation, contrast calculation, and accessibility checks
- **Multiple Formats**: Hex, RGB, RGBA with seamless conversion

### Typography
- **System Font Detection**: Platform-specific font selection (SF Pro, Segoe UI, Inter)
- **Consistent Scales**: Carefully designed font size and weight hierarchies
- **Text Style Presets**: Pre-defined styles for headings, body text, labels, and more
- **Compact Mode**: Dense typography for Unity-style interfaces

### Spacing & Layout
- **Consistent Spacing**: 4px-based spacing scale (xs: 4px, md: 16px, xl: 32px)
- **Component Sizing**: Standardized button, input, and panel dimensions
- **Layout Utilities**: Helper functions for margins, padding, and containers
- **Responsive Design**: Breakpoint system for different screen sizes

## 🧩 Components

### Base Components

#### EditorButton
```rust
use engine_ui_system::*;
use std::rc::Rc;
use std::cell::RefCell;

let theme = Rc::new(RefCell::new(EditorTheme::unity_dark()));

// Primary action button
let save_btn = EditorButton::primary("Save", theme.clone());

// Icon button for toolbar
let icon_btn = EditorButton::toolbar_icon("document-save", "Save File", theme.clone());

// Different variants
let secondary_btn = EditorButton::secondary("Cancel", theme.clone());
let danger_btn = EditorButton::danger("Delete", theme);
```

#### EditorInput
```rust
// Text input
let text_input = EditorInput::text_input(theme.clone());

// Specialized inputs
let search_input = EditorInput::search("Search...", theme.clone());
let number_input = EditorInput::number(theme.clone());
let password_input = EditorInput::password(theme);

// Numeric input with validation
let numeric = NumericInput::new(Some(0.0), Some(100.0), 1.0, theme);
```

#### EditorPanel
```rust
// Content panel with header
let panel = EditorPanel::content("Inspector", theme.clone());

// Specialized panels
let sidebar = EditorPanel::sidebar("Project", theme.clone());
let dialog = EditorPanel::dialog("Settings", theme);
```

#### EditorToolbar
```rust
// Standard editor toolbar
let toolbar = EditorToolbar::standard(theme.clone());

// Custom toolbar with tools
let mut toolbar = EditorToolbar::new(theme);
toolbar.add_tool(Tool::new("save", "document-save", "Save"));
toolbar.add_tool(Tool::new("undo", "edit-undo", "Undo"));
```

### Unity-Style Widgets

#### Vector3Field
```rust
// Position field
let position = Vector3Field::position(theme.clone());

// Custom vector field
let direction = Vector3Field::with_label("Direction", Vector3::forward(), theme);

// Vector2 for 2D properties
let size = Vector2Field::size(theme);
```

#### EnumDropdown
```rust
// Render mode dropdown
let render_mode = EnumDropdown::render_mode(theme.clone());

// Light type dropdown
let light_type = EnumDropdown::light_type(theme.clone());

// Custom enum
#[derive(Clone, PartialEq)]
enum Quality { Low, Medium, High }

impl EnumOption for Quality {
    fn display_name(&self) -> String { /* implementation */ }
    fn all_options() -> Vec<Self> { /* implementation */ }
}

let quality = EnumDropdown::with_default::<Quality>("Quality", theme);
```

#### AssetField
```rust
// Texture asset field
let albedo = AssetField::texture("Albedo", theme.clone());

// Material field
let material = AssetField::material("Material", theme.clone());

// Any asset type
let any_asset = AssetField::any("Asset", theme);
```

## 🎯 Usage Example

```rust
use engine_ui_system::*;
use std::rc::Rc;
use std::cell::RefCell;

fn create_inspector_ui() {
    // Create theme
    let theme = Rc::new(RefCell::new(EditorTheme::unity_dark()));
    
    // Create inspector panel
    let inspector = EditorPanel::content("Inspector", theme.clone());
    
    // Add transform section
    let position = Vector3Field::position(theme.clone());
    let rotation = Vector3Field::rotation(theme.clone());
    let scale = Vector3Field::scale(theme.clone());
    
    inspector.add_child_with_spacing(&position.as_widget());
    inspector.add_child_with_spacing(&rotation.as_widget());
    inspector.add_child_with_spacing(&scale.as_widget());
    
    // Add material section
    let material = AssetField::material("Material", theme.clone());
    let render_mode = EnumDropdown::render_mode(theme);
    
    inspector.add_child_with_spacing(&material.as_widget());
    inspector.add_child_with_spacing(&render_mode.as_widget());
}
```

## 🛠 Utilities

### CSS Generation
```rust
// Generate complete theme CSS
let theme = EditorTheme::unity_dark();
let css = utils::css::generate_theme_css(&theme);
utils::css::apply_global_css(&css)?;
```

### Color Utilities
```rust
// Color manipulation
let primary = Color::from_hex("#007AFF")?;
let lighter = primary.lighten(0.2);
let darker = primary.darken(0.1);

// Accessibility
let contrast = utils::color::contrast_ratio(&foreground, &background);
let accessible = utils::color::is_accessible(&text, &bg);
```

### Animation
```rust
// Easing functions
let easing = utils::animation::EasingFunction::EaseInOut;
let progress = easing.apply(0.5); // 0.5 for halfway

// Standard durations
use utils::animation::AnimationDurations;
// AnimationDurations::FAST   // 150ms
// AnimationDurations::NORMAL // 250ms
// AnimationDurations::SLOW   // 350ms
```

### Validation
```rust
// Input validation
let valid = utils::validation::is_valid_number("123.45");
let in_range = utils::validation::is_number_in_range("50", 0.0, 100.0);
let valid_hex = utils::validation::is_valid_hex_color("#FF0000");
```

## 🎨 Theme Customization

```rust
// Create custom theme
let custom_theme = EditorTheme::custom(
    "My Theme".to_string(),
    ColorPalette::unity_dark(),
    Typography::system_default(),
    Spacing::standard(),
    Sizes::compact(),
    Layout::standard(),
    true, // is_dark
);

// Theme manager
let mut theme_manager = ThemeManager::new();
theme_manager.add_theme(custom_theme);
theme_manager.switch_theme("My Theme")?;
```

## 📐 Design System

### Colors
- **Primary**: #007AFF (Unity blue)
- **Success**: #34C759 (green)
- **Warning**: #FF9500 (orange)
- **Error**: #FF3B30 (red)
- **Background Dark**: #1E1E1E
- **Background Light**: #FFFFFF

### Typography
- **macOS**: SF Pro Display, SF Mono
- **Windows**: Segoe UI, Consolas
- **Linux**: Inter, JetBrains Mono

### Spacing Scale
- **XS**: 4px - Minimal spacing
- **SM**: 8px - Small spacing
- **MD**: 16px - Default spacing
- **LG**: 24px - Large spacing
- **XL**: 32px - Extra large spacing
- **XXL**: 48px - Section spacing

### Component Sizes
- **Small**: 20-24px height
- **Medium**: 28-32px height (default)
- **Large**: 36-40px height

## 🧪 Testing

Run the comprehensive demo:
```bash
cargo run --package engine-ui-system --example theme_demo
```

Run unit tests:
```bash
cargo test --package engine-ui-system
```

## 🏗 Architecture

The unified component system is built with:
- **Separation of Concerns**: Theme, components, and utilities are separate modules
- **Consistent APIs**: All components follow the same patterns
- **GTK4 Integration**: Built on modern GTK4 widgets
- **Type Safety**: Rust's type system ensures correctness
- **Performance**: Efficient rendering and minimal allocations

## 📦 Dependencies

- **GTK4**: Modern UI toolkit
- **GDK4**: Graphics and input handling
- **libadwaita**: Modern GNOME design patterns
- **serde**: Serialization for themes and settings
- **csscolorparser**: Color parsing and manipulation

## 🎯 Goals Achieved

✅ **Unified Design Language**: Consistent components across the editor
✅ **Professional Appearance**: Unity-style polished interface
✅ **Theme Support**: Dark/light themes with easy switching
✅ **Developer Experience**: Simple, intuitive APIs
✅ **Performance**: Efficient rendering and minimal overhead
✅ **Accessibility**: WCAG AA compliant contrast ratios
✅ **Maintainability**: Clean, well-documented code structure

The unified component system provides everything needed to build a professional, Unity-style game editor interface with consistent theming, typography, and component behavior.