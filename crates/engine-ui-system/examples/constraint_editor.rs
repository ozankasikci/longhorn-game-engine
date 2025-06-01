// Interactive Constraint Editor
// Define exact specifications for UI elements

use engine_ui_system::design_constraints::*;
use std::io::{self, Write};
use std::process::Command;
use std::fs;

fn main() {
    println!("🎨 UI Design Constraint Editor");
    println!("==============================");
    println!("Define exact specifications for your UI elements");
    println!();

    let mut constraints = DesignConstraints::unity_dark();
    
    loop {
        show_menu();
        let choice = get_input("Enter your choice: ");
        
        match choice.trim() {
            "1" => edit_colors(&mut constraints),
            "2" => edit_typography(&mut constraints),
            "3" => edit_geometry(&mut constraints),
            "4" => edit_spacing(&mut constraints),
            "5" => edit_effects(&mut constraints),
            "6" => preview_constraints(&constraints),
            "7" => save_constraints(&constraints),
            "8" => load_presets(&mut constraints),
            "s" | "S" => search_and_edit(&mut constraints),
            "b" | "B" => {
                if build_and_start_engine(&constraints) {
                    break;
                }
            },
            "9" => break,
            _ => println!("❌ Invalid choice. Please try again."),
        }
        println!();
    }
}

fn show_menu() {
    println!("📐 What would you like to configure?");
    println!("  1. 🎨 Colors (backgrounds, buttons, text, borders)");
    println!("  2. 📝 Typography (fonts, sizes, weights)");
    println!("  3. 📏 Geometry (dimensions, roundness, borders)");
    println!("  4. 📍 Spacing (gaps, padding, margins)");
    println!("  5. ✨ Effects (shadows, animations, transitions)");
    println!("  6. 👁  Preview Current Settings");
    println!("  7. 💾 Save Configuration");
    println!("  8. 📁 Load Presets");
    println!("  s. 🔍 Search & Edit Individual Fields (Advanced)");
    println!("  b. 🚀 Build & Start Engine with Current Settings");
    println!("  9. 🚪 Exit");
    println!();
}

fn edit_colors(constraints: &mut DesignConstraints) {
    println!("🎨 Color Configuration");
    println!("======================");
    
    println!("Current colors:");
    println!("  Window Background: {}", constraints.colors.window_background);
    println!("  Panel Background: {}", constraints.colors.panel_background);
    println!("  Primary Button: {}", constraints.colors.button_primary_bg);
    println!("  Text Primary: {}", constraints.colors.text_primary);
    println!();
    
    println!("Which colors would you like to change?");
    println!("  1. Background Colors");
    println!("  2. Button Colors");
    println!("  3. Input Colors");
    println!("  4. Text Colors");
    println!("  5. Border Colors");
    println!("  6. Status Colors");
    
    let choice = get_input("Enter choice (1-6): ");
    
    match choice.trim() {
        "1" => {
            constraints.colors.window_background = get_color_input("Window Background", &constraints.colors.window_background);
            constraints.colors.panel_background = get_color_input("Panel Background", &constraints.colors.panel_background);
            constraints.colors.toolbar_background = get_color_input("Toolbar Background", &constraints.colors.toolbar_background);
            constraints.colors.sidebar_background = get_color_input("Sidebar Background", &constraints.colors.sidebar_background);
        },
        "2" => {
            constraints.colors.button_primary_bg = get_color_input("Primary Button", &constraints.colors.button_primary_bg);
            constraints.colors.button_primary_hover = get_color_input("Primary Button Hover", &constraints.colors.button_primary_hover);
            constraints.colors.button_secondary_bg = get_color_input("Secondary Button", &constraints.colors.button_secondary_bg);
            constraints.colors.button_danger_bg = get_color_input("Danger Button", &constraints.colors.button_danger_bg);
        },
        _ => println!("Feature coming soon..."),
    }
}

fn edit_typography(constraints: &mut DesignConstraints) {
    println!("📝 Typography Configuration");
    println!("===========================");
    
    println!("Current typography:");
    println!("  Primary Font: {}", constraints.typography.primary_font);
    println!("  Base Size: {}px", constraints.typography.font_size_base);
    println!("  Normal Weight: {}", constraints.typography.weight_normal);
    println!();
    
    constraints.typography.primary_font = get_input_default("Primary Font", &constraints.typography.primary_font);
    constraints.typography.monospace_font = get_input_default("Monospace Font", &constraints.typography.monospace_font);
    
    if let Ok(size) = get_input("Base Font Size (px): ").trim().parse::<f32>() {
        constraints.typography.font_size_base = size;
        // Scale other sizes proportionally
        constraints.typography.font_size_xs = size - 2.0;
        constraints.typography.font_size_sm = size - 1.0;
        constraints.typography.font_size_md = size + 1.0;
        constraints.typography.font_size_lg = size + 2.0;
        constraints.typography.font_size_xl = size + 4.0;
    }
}

fn edit_geometry(constraints: &mut DesignConstraints) {
    println!("📏 Geometry Configuration");
    println!("=========================");
    
    println!("Current geometry:");
    println!("  Button Height: {}px", constraints.geometry.button_height_md);
    println!("  Border Radius: {}px", constraints.geometry.border_radius_md);
    println!("  Border Width: {}px", constraints.geometry.border_width_thin);
    println!();
    
    if let Ok(height) = get_input("Button Height (px): ").trim().parse::<f32>() {
        constraints.geometry.button_height_md = height;
        constraints.geometry.button_height_sm = height - 4.0;
        constraints.geometry.button_height_lg = height + 4.0;
        constraints.geometry.input_height_md = height;
        constraints.geometry.input_height_sm = height - 4.0;
        constraints.geometry.input_height_lg = height + 4.0;
    }
    
    if let Ok(radius) = get_input("Border Radius (px, 0 = flat): ").trim().parse::<f32>() {
        constraints.geometry.border_radius_md = radius;
        constraints.geometry.border_radius_sm = (radius * 0.5).max(0.0);
        constraints.geometry.border_radius_lg = radius * 1.5;
        constraints.geometry.border_radius_xl = radius * 2.0;
    }
    
    if let Ok(width) = get_input("Border Width (px): ").trim().parse::<f32>() {
        constraints.geometry.border_width_thin = width;
        constraints.geometry.border_width_thick = width * 2.0;
    }
}

fn edit_spacing(constraints: &mut DesignConstraints) {
    println!("📍 Spacing Configuration");
    println!("========================");
    
    println!("Current spacing:");
    println!("  Base Unit: {}px", constraints.spacing.space_md);
    println!("  Panel Padding: {}px", constraints.spacing.panel_padding);
    println!();
    
    if let Ok(base) = get_input("Base Spacing Unit (px): ").trim().parse::<f32>() {
        constraints.spacing.space_md = base;
        constraints.spacing.space_xs = base * 0.25;
        constraints.spacing.space_sm = base * 0.5;
        constraints.spacing.space_lg = base * 1.5;
        constraints.spacing.space_xl = base * 2.0;
        constraints.spacing.space_xxl = base * 3.0;
        constraints.spacing.space_xxxl = base * 4.0;
        
        constraints.spacing.button_gap = base;
        constraints.spacing.panel_padding = base * 1.5;
        constraints.spacing.section_spacing = base * 2.0;
        constraints.spacing.field_spacing = base;
        constraints.spacing.toolbar_item_spacing = base * 0.5;
    }
}

fn edit_effects(constraints: &mut DesignConstraints) {
    println!("✨ Effects Configuration");
    println!("========================");
    
    println!("Current effects:");
    println!("  Shadow Blur: {}px", constraints.effects.shadow_md_blur);
    println!("  Animation Speed: {}ms", constraints.effects.animation_normal);
    println!();
    
    println!("Shadow style:");
    println!("  1. No shadows (flat)");
    println!("  2. Subtle shadows");
    println!("  3. Prominent shadows");
    
    let choice = get_input("Enter choice (1-3): ");
    match choice.trim() {
        "1" => {
            constraints.effects.shadow_sm_blur = 0.0;
            constraints.effects.shadow_md_blur = 0.0;
            constraints.effects.shadow_lg_blur = 0.0;
        },
        "2" => {
            constraints.effects.shadow_sm_blur = 1.0;
            constraints.effects.shadow_md_blur = 2.0;
            constraints.effects.shadow_lg_blur = 4.0;
        },
        "3" => {
            constraints.effects.shadow_sm_blur = 4.0;
            constraints.effects.shadow_md_blur = 8.0;
            constraints.effects.shadow_lg_blur = 16.0;
        },
        _ => {}
    }
    
    if let Ok(speed) = get_input("Animation Speed (ms): ").trim().parse::<f32>() {
        constraints.effects.animation_normal = speed;
        constraints.effects.animation_fast = speed * 0.5;
        constraints.effects.animation_slow = speed * 1.5;
        constraints.effects.hover_transition = speed * 0.75;
        constraints.effects.focus_transition = speed * 0.5;
    }
}

fn preview_constraints(constraints: &DesignConstraints) {
    println!("👁  Current Design Constraints");
    println!("==============================");
    
    println!("🎨 Colors:");
    println!("  Window: {} | Panel: {} | Toolbar: {}", 
        constraints.colors.window_background,
        constraints.colors.panel_background,
        constraints.colors.toolbar_background);
    println!("  Primary Button: {} | Text: {}", 
        constraints.colors.button_primary_bg,
        constraints.colors.text_primary);
    
    println!("\n📝 Typography:");
    println!("  Font: {} | Size: {}px | Weight: {}", 
        constraints.typography.primary_font,
        constraints.typography.font_size_base,
        constraints.typography.weight_normal);
    
    println!("\n📏 Geometry:");
    println!("  Button: {}px high | Radius: {}px | Border: {}px", 
        constraints.geometry.button_height_md,
        constraints.geometry.border_radius_md,
        constraints.geometry.border_width_thin);
    
    println!("\n📍 Spacing:");
    println!("  Base Unit: {}px | Panel Padding: {}px", 
        constraints.spacing.space_md,
        constraints.spacing.panel_padding);
    
    println!("\n✨ Effects:");
    println!("  Shadow Blur: {}px | Animation: {}ms", 
        constraints.effects.shadow_md_blur,
        constraints.effects.animation_normal);
    
    println!("\n💅 Generated CSS Preview:");
    println!("{}", constraints.to_css());
}

fn save_constraints(constraints: &DesignConstraints) {
    let json = serde_json::to_string_pretty(constraints).unwrap();
    println!("💾 Configuration JSON:");
    println!("{}", json);
    println!("\n✅ Copy this JSON to save your constraints!");
}

fn load_presets(constraints: &mut DesignConstraints) {
    println!("📁 Available Presets:");
    println!("  1. Unity Dark (current default)");
    println!("  2. Flat Minimal (no shadows, sharp corners)");
    println!("  3. VSCode Style");
    println!("  4. Blender Style");
    
    let choice = get_input("Enter choice (1-4): ");
    
    match choice.trim() {
        "1" => *constraints = DesignConstraints::unity_dark(),
        "2" => *constraints = DesignConstraints::flat_minimal(),
        "3" => {
            *constraints = DesignConstraints::unity_dark();
            // VSCode-like modifications
            constraints.colors.window_background = "#1E1E1E".to_string();
            constraints.colors.panel_background = "#252526".to_string();
            constraints.colors.toolbar_background = "#2D2D30".to_string();
            constraints.colors.button_primary_bg = "#0E639C".to_string();
            constraints.geometry.border_radius_md = 2.0;
        },
        "4" => {
            *constraints = DesignConstraints::unity_dark();
            // Blender-like modifications
            constraints.colors.window_background = "#383838".to_string();
            constraints.colors.panel_background = "#484848".to_string();
            constraints.colors.button_primary_bg = "#5D87A1".to_string();
            constraints.geometry.border_radius_md = 3.0;
            constraints.spacing.space_md = 6.0;
        },
        _ => println!("❌ Invalid preset choice"),
    }
    
    println!("✅ Preset loaded!");
}

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn get_input_default(prompt: &str, default: &str) -> String {
    let full_prompt = format!("{} [{}]: ", prompt, default);
    let input = get_input(&full_prompt);
    let trimmed = input.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

fn get_color_input(prompt: &str, current: &str) -> String {
    println!("\n🎨 {} Color Picker", prompt);
    println!("Current: {}", current);
    println!();
    
    // Show color presets
    println!("🎯 Quick Presets:");
    println!("  1. #1E1E1E (VS Code Dark)    2. #252526 (VS Code Panel)   3. #2D2D30 (VS Code Menu)");
    println!("  4. #383838 (Unity Dark)      5. #484848 (Unity Panel)     6. #404040 (Unity Toolbar)");
    println!("  7. #4A90E2 (Blue Primary)    8. #5BA0F2 (Blue Hover)      9. #E74C3C (Red Danger)");
    println!(" 10. #27AE60 (Green Success)  11. #F39C12 (Orange Warning)  12. #9B59B6 (Purple Accent)");
    println!(" 13. #FFFFFF (Pure White)     14. #000000 (Pure Black)     15. #CCCCCC (Light Gray)");
    println!(" 16. #666666 (Medium Gray)    17. #333333 (Dark Gray)      18. #95A5A6 (Muted Gray)");
    println!();
    
    // Show named color options
    println!("🏷  Named Colors: red, green, blue, yellow, purple, orange, pink, cyan, white, black, gray");
    println!("📝 Custom Hex: Enter any #RRGGBB format (e.g., #FF5733)");
    println!("⏎  Keep Current: Press Enter to keep current color");
    println!();
    
    let input = get_input("Your choice (number, name, or hex): ");
    let trimmed = input.trim().to_lowercase();
    
    if trimmed.is_empty() {
        return current.to_string();
    }
    
    // Handle numbered presets
    match trimmed.as_str() {
        "1" => return "#1E1E1E".to_string(),
        "2" => return "#252526".to_string(),
        "3" => return "#2D2D30".to_string(),
        "4" => return "#383838".to_string(),
        "5" => return "#484848".to_string(),
        "6" => return "#404040".to_string(),
        "7" => return "#4A90E2".to_string(),
        "8" => return "#5BA0F2".to_string(),
        "9" => return "#E74C3C".to_string(),
        "10" => return "#27AE60".to_string(),
        "11" => return "#F39C12".to_string(),
        "12" => return "#9B59B6".to_string(),
        "13" => return "#FFFFFF".to_string(),
        "14" => return "#000000".to_string(),
        "15" => return "#CCCCCC".to_string(),
        "16" => return "#666666".to_string(),
        "17" => return "#333333".to_string(),
        "18" => return "#95A5A6".to_string(),
        _ => {}
    }
    
    // Handle named colors
    let named_color = match trimmed.as_str() {
        "red" => Some("#E74C3C"),
        "green" => Some("#27AE60"),
        "blue" => Some("#4A90E2"),
        "yellow" => Some("#F1C40F"),
        "purple" => Some("#9B59B6"),
        "orange" => Some("#F39C12"),
        "pink" => Some("#E91E63"),
        "cyan" => Some("#1ABC9C"),
        "white" => Some("#FFFFFF"),
        "black" => Some("#000000"),
        "gray" | "grey" => Some("#95A5A6"),
        "lightgray" | "lightgrey" => Some("#CCCCCC"),
        "darkgray" | "darkgrey" => Some("#333333"),
        _ => None,
    };
    
    if let Some(color) = named_color {
        println!("✅ Applied {} color: {}", trimmed, color);
        return color.to_string();
    }
    
    // Handle hex input
    if trimmed.starts_with('#') && trimmed.len() == 7 {
        // Validate hex format
        if trimmed.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
            println!("✅ Applied custom color: {}", trimmed.to_uppercase());
            return trimmed.to_uppercase();
        }
    }
    
    println!("❌ Invalid color format. Using current value: {}", current);
    current.to_string()
}

fn build_and_start_engine(constraints: &DesignConstraints) -> bool {
    println!("🚀 Building & Starting Engine");
    println!("=============================");
    
    // Save current constraints to a config file
    println!("📝 Saving design constraints...");
    let config_json = serde_json::to_string_pretty(constraints).unwrap();
    let config_path = "target/current_design_constraints.json";
    
    if let Err(e) = fs::write(config_path, &config_json) {
        println!("❌ Failed to save constraints: {}", e);
        return false;
    }
    
    println!("✅ Constraints saved to: {}", config_path);
    
    // Generate CSS file
    println!("🎨 Generating CSS from constraints...");
    let css = constraints.to_css();
    let css_path = "target/current_design.css";
    
    if let Err(e) = fs::write(css_path, &css) {
        println!("❌ Failed to save CSS: {}", e);
        return false;
    }
    
    println!("✅ CSS generated: {}", css_path);
    
    // Show what we're about to build
    println!("\n🎯 Design Summary:");
    println!("  Window: {} | Button: {} | Text: {}", 
        constraints.colors.window_background,
        constraints.colors.button_primary_bg,
        constraints.colors.text_primary);
    println!("  Button Height: {}px | Radius: {}px | Spacing: {}px", 
        constraints.geometry.button_height_md,
        constraints.geometry.border_radius_md,
        constraints.spacing.space_md);
    
    
    println!("\n🔨 Building engine...");
    
    // Build the engine-editor package
    let build_result = Command::new("cargo")
        .args(&["build", "--package", "engine-editor", "--release"])
        .status();
    
    match build_result {
        Ok(status) if status.success() => {
            println!("✅ Build successful!");
        },
        Ok(status) => {
            println!("❌ Build failed with exit code: {}", status.code().unwrap_or(-1));
            println!("💡 Try fixing any compilation errors and run 'b' again.");
            return false;
        },
        Err(e) => {
            println!("❌ Failed to run build command: {}", e);
            return false;
        }
    }
    
    println!("\n🚀 Starting engine with your custom design...");
    println!("💡 The editor will use the constraints you just defined!");
    println!("🔄 Close the editor to return to the constraint editor.");
    
    // Start the engine
    let run_result = Command::new("cargo")
        .args(&["run", "--package", "engine-editor", "--release"])
        .status();
    
    match run_result {
        Ok(_) => {
            println!("\n🔙 Editor closed. Welcome back to constraint editor!");
            false // Don't exit constraint editor
        },
        Err(e) => {
            println!("❌ Failed to start engine: {}", e);
            false
        }
    }
}

fn create_constraints_integration_code(constraints: &DesignConstraints) -> String {
    format!(
        r#"
// Auto-generated design constraints integration
// This code applies your custom design constraints to the editor

use engine_ui_system::{{DesignConstraints, utils}};

pub fn apply_custom_design() -> DesignConstraints {{
    let constraints_json = include_str!("../target/current_design_constraints.json");
    
    match serde_json::from_str(constraints_json) {{
        Ok(constraints) => {{
            // Apply the CSS globally
            let css = constraints.to_css();
            if let Err(e) = utils::css::apply_global_css(&css) {{
                eprintln!("Warning: Failed to apply custom CSS: {{}}", e);
            }} else {{
                println!("✅ Applied custom design constraints");
            }}
            constraints
        }},
        Err(e) => {{
            eprintln!("Warning: Failed to load custom constraints: {{}}", e);
            eprintln!("Using default Unity Dark theme");
            DesignConstraints::unity_dark()
        }}
    }}
}}
"#
    )
}

fn search_and_edit(constraints: &mut DesignConstraints) {
    println!("🔍 Search & Edit Individual Fields");
    println!("==================================");
    
    let search_term = get_input("Enter search term (partial field name): ");
    let search_term = search_term.trim().to_lowercase();
    
    if search_term.is_empty() {
        println!("❌ Search term cannot be empty");
        return;
    }
    
    let matches = find_matching_fields(&search_term);
    
    if matches.is_empty() {
        println!("❌ No fields found matching '{}'", search_term);
        println!("💡 Try broader terms like 'color', 'font', 'space', 'shadow', etc.");
        return;
    }
    
    println!("\n📋 Found {} matching field(s):", matches.len());
    for (i, (path, description, current_value)) in matches.iter().enumerate() {
        println!("  {}. {} - {} (current: {})", i + 1, path, description, current_value);
    }
    
    let choice = get_input("\nEnter field number to edit (or 0 to cancel): ");
    
    if let Ok(index) = choice.trim().parse::<usize>() {
        if index == 0 {
            return;
        }
        if index > 0 && index <= matches.len() {
            let (field_path, description, current_value) = &matches[index - 1];
            edit_field_by_path(constraints, field_path, description, current_value);
        } else {
            println!("❌ Invalid field number");
        }
    } else {
        println!("❌ Invalid input");
    }
}

fn find_matching_fields(search_term: &str) -> Vec<(String, String, String)> {
    let mut matches = Vec::new();
    
    // Color fields
    if search_term.contains("color") || search_term.contains("bg") || search_term.contains("background") || search_term.contains("window") {
        matches.push(("colors.window_background".to_string(), "Main window background color".to_string(), "#2E2E2E".to_string()));
        matches.push(("colors.panel_background".to_string(), "Panel background color".to_string(), "#383838".to_string()));
        matches.push(("colors.toolbar_background".to_string(), "Toolbar background color".to_string(), "#404040".to_string()));
    }
    
    if search_term.contains("button") || search_term.contains("primary") {
        matches.push(("colors.button_primary_bg".to_string(), "Primary button background".to_string(), "#4A90E2".to_string()));
        matches.push(("colors.button_primary_hover".to_string(), "Primary button hover color".to_string(), "#5BA0F2".to_string()));
    }
    
    if search_term.contains("text") || search_term.contains("font") {
        matches.push(("colors.text_primary".to_string(), "Primary text color".to_string(), "#FFFFFF".to_string()));
        matches.push(("colors.text_secondary".to_string(), "Secondary text color".to_string(), "#CCCCCC".to_string()));
    }
    
    // Typography fields
    if search_term.contains("font") || search_term.contains("typography") || search_term.contains("text") {
        matches.push(("typography.primary_font".to_string(), "Primary font family".to_string(), "Inter".to_string()));
        matches.push(("typography.font_size_base".to_string(), "Base font size in pixels".to_string(), "12.0".to_string()));
    }
    
    // Geometry fields
    if search_term.contains("height") || search_term.contains("button") || search_term.contains("geometry") {
        matches.push(("geometry.button_height_md".to_string(), "Medium button height".to_string(), "24.0".to_string()));
    }
    
    if search_term.contains("radius") || search_term.contains("border") || search_term.contains("round") {
        matches.push(("geometry.border_radius_md".to_string(), "Medium border radius".to_string(), "4.0".to_string()));
    }
    
    // Spacing fields
    if search_term.contains("space") || search_term.contains("spacing") || search_term.contains("gap") || search_term.contains("padding") {
        matches.push(("spacing.space_md".to_string(), "Medium spacing unit".to_string(), "8.0".to_string()));
        matches.push(("spacing.panel_padding".to_string(), "Panel padding".to_string(), "12.0".to_string()));
    }
    
    // Effects fields
    if search_term.contains("shadow") || search_term.contains("blur") || search_term.contains("effect") {
        matches.push(("effects.shadow_md_blur".to_string(), "Medium shadow blur radius".to_string(), "4.0".to_string()));
    }
    
    if search_term.contains("animation") || search_term.contains("speed") || search_term.contains("transition") {
        matches.push(("effects.animation_normal".to_string(), "Normal animation duration (ms)".to_string(), "200.0".to_string()));
    }
    
    matches
}

fn edit_field_by_path(constraints: &mut DesignConstraints, field_path: &str, description: &str, current_value: &str) {
    println!("\n✏️  Editing: {}", description);
    println!("📍 Field: {}", field_path);
    println!("🔄 Current: {}", current_value);
    
    let new_value = get_input_default("New value", current_value);
    
    // Apply the change based on field path
    match field_path {
        "colors.window_background" => constraints.colors.window_background = new_value,
        "colors.panel_background" => constraints.colors.panel_background = new_value,
        "colors.toolbar_background" => constraints.colors.toolbar_background = new_value,
        "colors.button_primary_bg" => constraints.colors.button_primary_bg = new_value,
        "colors.button_primary_hover" => constraints.colors.button_primary_hover = new_value,
        "colors.text_primary" => constraints.colors.text_primary = new_value,
        "colors.text_secondary" => constraints.colors.text_secondary = new_value,
        "typography.primary_font" => constraints.typography.primary_font = new_value,
        "typography.font_size_base" => {
            if let Ok(size) = new_value.parse::<f32>() {
                constraints.typography.font_size_base = size;
                println!("✅ Font size updated to {}px", size);
            } else {
                println!("❌ Invalid number format");
            }
        },
        "geometry.button_height_md" => {
            if let Ok(height) = new_value.parse::<f32>() {
                constraints.geometry.button_height_md = height;
                println!("✅ Button height updated to {}px", height);
            } else {
                println!("❌ Invalid number format");
            }
        },
        "geometry.border_radius_md" => {
            if let Ok(radius) = new_value.parse::<f32>() {
                constraints.geometry.border_radius_md = radius;
                println!("✅ Border radius updated to {}px", radius);
            } else {
                println!("❌ Invalid number format");
            }
        },
        "spacing.space_md" => {
            if let Ok(space) = new_value.parse::<f32>() {
                constraints.spacing.space_md = space;
                println!("✅ Spacing updated to {}px", space);
            } else {
                println!("❌ Invalid number format");
            }
        },
        "spacing.panel_padding" => {
            if let Ok(padding) = new_value.parse::<f32>() {
                constraints.spacing.panel_padding = padding;
                println!("✅ Panel padding updated to {}px", padding);
            } else {
                println!("❌ Invalid number format");
            }
        },
        "effects.shadow_md_blur" => {
            if let Ok(blur) = new_value.parse::<f32>() {
                constraints.effects.shadow_md_blur = blur;
                println!("✅ Shadow blur updated to {}px", blur);
            } else {
                println!("❌ Invalid number format");
            }
        },
        "effects.animation_normal" => {
            if let Ok(duration) = new_value.parse::<f32>() {
                constraints.effects.animation_normal = duration;
                println!("✅ Animation duration updated to {}ms", duration);
            } else {
                println!("❌ Invalid number format");
            }
        },
        _ => {
            println!("❌ Unknown field path: {}", field_path);
        }
    }
}