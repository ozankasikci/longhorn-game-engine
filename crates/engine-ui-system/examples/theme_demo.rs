// Demo of the unified component system
// This demonstrates the theme, colors, typography, and components working together

use engine_ui_system::*;
use engine_ui_system::widgets::enum_dropdown::EnumOption;

fn main() {
    println!("🎨 Unified Component System Demo");
    println!("=====================================");
    
    // Create theme manager
    let mut theme_manager = ThemeManager::new();
    
    // Show available themes
    println!("\n📋 Available Themes:");
    for name in theme_manager.theme_names() {
        println!("  - {}", name);
    }
    
    // Get current theme
    let current_theme = theme_manager.current_theme();
    println!("\n🌟 Current Theme: {}", current_theme.name);
    
    // Demonstrate color system
    println!("\n🎨 Color Palette:");
    println!("  Primary: {}", current_theme.colors.primary);
    println!("  Background: {}", current_theme.colors.background);
    println!("  Text: {}", current_theme.colors.text_primary);
    
    // Demonstrate typography
    println!("\n📝 Typography:");
    println!("  Font Family: {}", current_theme.typography.font_family_primary);
    println!("  Base Size: {}px", current_theme.typography.sizes.base);
    
    // Demonstrate spacing
    println!("\n📏 Spacing Scale:");
    println!("  XS: {}px", current_theme.spacing.xs);
    println!("  MD: {}px", current_theme.spacing.md);
    println!("  XL: {}px", current_theme.spacing.xl);
    
    // Demonstrate sizes
    println!("\n📐 Component Sizes:");
    println!("  Button Height (MD): {}px", current_theme.sizes.button_height_md);
    println!("  Input Height (MD): {}px", current_theme.sizes.input_height_md);
    println!("  Border Radius: {}px", current_theme.sizes.border_radius_md);
    
    // Demonstrate CSS generation
    println!("\n💅 CSS Generation:");
    let button_css = current_theme.button_css("primary", "medium");
    println!("  Button CSS snippet: {}", button_css.lines().next().unwrap_or("").trim());
    
    // Toggle theme
    println!("\n🔄 Toggling Dark Mode...");
    if let Err(e) = theme_manager.toggle_dark_mode() {
        println!("  Error: {}", e);
    } else {
        let new_theme = theme_manager.current_theme();
        println!("  New Theme: {}", new_theme.name);
        println!("  New Background: {}", new_theme.colors.background);
    }
    
    // Demonstrate Vector3 types
    println!("\n🧮 Vector3 System:");
    let position = widgets::vector_field::Vector3::new(1.0, 2.0, 3.0);
    println!("  Position: {}", position);
    
    let zero = widgets::vector_field::Vector3::zero();
    println!("  Zero: {}", zero);
    
    // Demonstrate asset types
    println!("\n📁 Asset System:");
    let texture_type = widgets::asset_field::AssetType::texture();
    println!("  Texture Extensions: {:?}", texture_type.extensions);
    
    let audio_type = widgets::asset_field::AssetType::audio();
    println!("  Audio Extensions: {:?}", audio_type.extensions);
    
    // Demonstrate enum options
    println!("\n🔽 Enum System:");
    let render_modes = widgets::enum_dropdown::RenderMode::all_options();
    println!("  Render Modes: {:?}", render_modes.iter().map(|r| r.display_name()).collect::<Vec<_>>());
    
    // Demonstrate utility functions
    println!("\n🛠  Utilities:");
    
    // Color utilities
    let black = Color::BLACK;
    let white = Color::WHITE;
    let contrast = utils::color::contrast_ratio(&black, &white);
    println!("  Black/White Contrast Ratio: {:.1}:1", contrast);
    
    let is_accessible = utils::color::is_accessible(&black, &white);
    println!("  Meets Accessibility Standards: {}", is_accessible);
    
    // Animation utilities
    let easing = utils::animation::EasingFunction::EaseInOut;
    let halfway = easing.apply(0.5);
    println!("  EaseInOut at 50%: {:.3}", halfway);
    
    // Validation utilities
    let valid_number = utils::validation::is_valid_number("123.45");
    let invalid_number = utils::validation::is_valid_number("abc");
    println!("  '123.45' is valid number: {}", valid_number);
    println!("  'abc' is valid number: {}", invalid_number);
    
    println!("\n✅ Demo completed successfully!");
    println!("   The unified component system is ready for use.");
}