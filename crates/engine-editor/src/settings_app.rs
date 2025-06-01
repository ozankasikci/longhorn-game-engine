use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, Button, ColorButton, Label, Entry, Orientation, Frame, Grid, Popover, FlowBox, DrawingArea, ColorChooserDialog, ResponseType};
use engine_ui_system::{DesignConstraints, design_loader, EditorTheme, EditorButton, EditorInput};
use std::rc::Rc;
use std::cell::RefCell;

const APP_ID: &str = "com.mobilegameengine.settings";

// Create a custom instant color picker - no dialogs, just click and apply!
fn create_custom_color_picker(initial_color: &str, label_text: &str, save_callback: impl Fn(String) + 'static + Clone) -> Box {
    let container = Box::new(Orientation::Horizontal, 8);
    
    // Current color display button
    let current_color_btn = Button::new();
    current_color_btn.set_size_request(50, 30);
    
    // Set initial color
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&format!(
        "button {{ background: {}; border: 2px solid #666; border-radius: 4px; min-width: 50px; min-height: 30px; }}",
        initial_color
    ));
    current_color_btn.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
    // Color palette
    let palette_container = Box::new(Orientation::Vertical, 4);
    
    // Common UI colors for game engine
    let color_rows = [
        // Grays and Darks
        ["#000000", "#1a1a1a", "#2d2d2d", "#3a3a3a", "#4a4a4a", "#5a5a5a", "#6a6a6a", "#808080"],
        // Light grays and whites  
        ["#999999", "#b3b3b3", "#cccccc", "#e6e6e6", "#f0f0f0", "#f8f8f8", "#ffffff", "#fafafa"],
        // Blues
        ["#001133", "#003366", "#0066cc", "#3399ff", "#66b3ff", "#99ccff", "#cce6ff", "#e6f3ff"],
        // Greens
        ["#001a00", "#003300", "#006600", "#009900", "#33cc33", "#66ff66", "#99ff99", "#ccffcc"],
        // Reds
        ["#330000", "#660000", "#990000", "#cc0000", "#ff3333", "#ff6666", "#ff9999", "#ffcccc"],
        // Yellows/Oranges
        ["#332200", "#664400", "#996600", "#cc8800", "#ffaa00", "#ffcc33", "#ffdd66", "#ffee99"],
        // Purples
        ["#220033", "#440066", "#660099", "#8800cc", "#aa33ff", "#cc66ff", "#dd99ff", "#eeccff"],
        // UI Accent Colors
        ["#007AFF", "#34C759", "#FF9500", "#FF3B30", "#AF52DE", "#FF2D92", "#5AC8FA", "#FFCC00"],
    ];
    
    for row in color_rows {
        let row_box = Box::new(Orientation::Horizontal, 2);
        
        for &color in &row {
            let color_btn = Button::new();
            color_btn.set_size_request(24, 24);
            
            let css_provider = gtk4::CssProvider::new();
            css_provider.load_from_data(&format!(
                "button {{ background: {}; border: 1px solid #333; border-radius: 3px; min-width: 24px; min-height: 24px; }}",
                color
            ));
            color_btn.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            
            // Apply color instantly when clicked
            let color_str = color.to_string();
            let save_callback = save_callback.clone();
            let current_color_btn_weak = current_color_btn.downgrade();
            
            color_btn.connect_clicked(move |_| {
                println!("🎨 Instantly applying color: {}", color_str);
                save_callback(color_str.clone());
                
                // Update current color display
                if let Some(current_btn) = current_color_btn_weak.upgrade() {
                    let css_provider = gtk4::CssProvider::new();
                    css_provider.load_from_data(&format!(
                        "button {{ background: {}; border: 2px solid #666; border-radius: 4px; min-width: 50px; min-height: 30px; }}",
                        color_str
                    ));
                    current_btn.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                }
            });
            
            row_box.append(&color_btn);
        }
        
        palette_container.append(&row_box);
    }
    
    // Create popover for palette
    let popover = Popover::new();
    popover.set_child(Some(&palette_container));
    popover.set_parent(&current_color_btn);
    popover.set_position(gtk4::PositionType::Bottom);
    
    // Show palette when current color button is clicked
    current_color_btn.connect_clicked(move |_| {
        popover.popup();
    });
    
    // Add label
    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);
    label.set_size_request(120, -1);
    
    container.append(&label);
    container.append(&current_color_btn);
    
    container
}

// Create an instant color picker that applies colors immediately when clicked (alternative implementation)
fn create_instant_color_picker(initial_color: &str, save_callback: impl Fn(String) + 'static + Clone) -> Button {
    let color_button = Button::new();
    color_button.set_size_request(40, 30);
    
    // Set initial color as background
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&format!(
        "button {{ background: {}; border: 2px solid #ccc; border-radius: 4px; }}",
        initial_color
    ));
    color_button.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
    // Create color palette
    let popover = Popover::new();
    let palette_box = FlowBox::new();
    palette_box.set_max_children_per_line(8);
    palette_box.set_column_spacing(4);
    palette_box.set_row_spacing(4);
    palette_box.set_margin_top(10);
    palette_box.set_margin_bottom(10);
    palette_box.set_margin_start(10);
    palette_box.set_margin_end(10);
    
    // Add predefined color palette
    let colors = [
        "#1e1e1e", "#2d2d2d", "#3a3a3a", "#4a4a4a", "#5a5a5a", "#6a6a6a", "#7a7a7a", "#8a8a8a",
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#E74C3C", "#3498DB", "#2ECC71", "#F39C12", "#9B59B6", "#1ABC9C", "#E67E22", "#34495E",
        "#C0392B", "#2980B9", "#27AE60", "#F1C40F", "#8E44AD", "#16A085", "#D35400", "#2C3E50",
        "#000000", "#333333", "#666666", "#999999", "#CCCCCC", "#FFFFFF", "#FF0000", "#00FF00",
    ];
    
    for color in &colors {
        let color_swatch = Button::new();
        color_swatch.set_size_request(32, 32);
        
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(&format!(
            "button {{ background: {}; border: 1px solid #333; border-radius: 4px; min-width: 32px; min-height: 32px; }}",
            color
        ));
        color_swatch.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        
        let color_str = color.to_string();
        let save_callback = save_callback.clone();
        let color_button_ref = color_button.downgrade();
        let popover_ref = popover.downgrade();
        
        color_swatch.connect_clicked(move |_| {
            // Apply color immediately
            println!("🎨 Instantly applying color: {}", color_str);
            save_callback(color_str.clone());
            
            // Update button appearance
            if let Some(button) = color_button_ref.upgrade() {
                let css_provider = gtk4::CssProvider::new();
                css_provider.load_from_data(&format!(
                    "button {{ background: {}; border: 2px solid #ccc; border-radius: 4px; }}",
                    color_str
                ));
                button.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            }
            
            // Close popover
            if let Some(popover) = popover_ref.upgrade() {
                popover.popdown();
            }
        });
        
        palette_box.insert(&color_swatch, -1);
    }
    
    popover.set_child(Some(&palette_box));
    popover.set_parent(&color_button);
    
    // Show popover when button is clicked
    color_button.connect_clicked(move |_| {
        popover.popup();
    });
    
    color_button
}

// Helper function to save constraints to file immediately
fn save_constraints_to_file(constraints: &DesignConstraints) {
    if let Err(e) = std::fs::write("target/current_design_constraints.json", serde_json::to_string_pretty(constraints).unwrap()) {
        eprintln!("❌ Failed to save settings: {}", e);
    } else {
        println!("🔥 Hot reload: Settings saved and applied instantly!");
    }
}

// Note: Individual input fields now have instant save via connect_changed callbacks
// This provides better user experience with immediate feedback

pub fn run_settings_app() -> glib::ExitCode {
    let application = Application::builder()
        .application_id(APP_ID)
        .build();

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Engine Settings")
        .default_width(800)
        .default_height(900)
        .resizable(true)
        .build();

    // Load current constraints
    let constraints = design_loader::load_current_design();

    // Create theme for all UI components
    let theme = Rc::new(RefCell::new(EditorTheme::default()));

    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Title
    let title = Label::new(Some("Engine Design Settings"));
    title.set_markup("<span size='x-large' weight='bold'>Engine Design Settings</span>");
    title.set_halign(gtk4::Align::Center);
    main_box.append(&title);

    // General Settings Section (Most Common)
    let general_frame = Frame::new(Some("General Settings"));
    let general_grid = Grid::new();
    general_grid.set_margin_top(10);
    general_grid.set_margin_bottom(10);
    general_grid.set_margin_start(10);
    general_grid.set_margin_end(10);
    general_grid.set_row_spacing(10);
    general_grid.set_column_spacing(15);

    // Most commonly used settings - using our custom color picker
    let window_color_picker = create_custom_color_picker(&constraints.colors.window_background, "Window Background:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.window_background = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let primary_color_picker = create_custom_color_picker(&constraints.colors.button_primary_bg, "Primary Button Color:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.button_primary_bg = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let text_color_picker = create_custom_color_picker(&constraints.colors.text_primary, "Text Color:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.text_primary = color;
            save_constraints_to_file(&constraints);
        }
    });
    let button_height_md_entry = EditorInput::number(theme.clone());
    let space_md_entry = EditorInput::number(theme.clone());
    let font_family_entry = EditorInput::text_input(theme.clone());
    let base_font_size_entry = EditorInput::number(theme.clone());

    // Custom color pickers handle their own initial values
    button_height_md_entry.set_text(&constraints.geometry.button_height_md.to_string());
    space_md_entry.set_text(&constraints.spacing.space_md.to_string());
    font_family_entry.set_text(&constraints.typography.primary_font);
    base_font_size_entry.set_text(&constraints.typography.font_size_base.to_string());

    // Connect instant save for numeric entries
    button_height_md_entry.connect_changed(move |text| {
        if let Ok(height) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.button_height_md = height;
            save_constraints_to_file(&constraints);
            println!("🎨 Button height updated instantly: {}px", height);
        }
    });

    space_md_entry.connect_changed(move |text| {
        if let Ok(spacing) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.spacing.space_md = spacing;
            save_constraints_to_file(&constraints);
            println!("🎨 Spacing updated instantly: {}px", spacing);
        }
    });

    font_family_entry.connect_changed(move |text| {
        if !text.is_empty() {
            let mut constraints = design_loader::load_current_design();
            constraints.typography.primary_font = text.to_string();
            save_constraints_to_file(&constraints);
            println!("🎨 Font family updated: {}", text);
        }
    });

    base_font_size_entry.connect_changed(move |text| {
        if let Ok(size) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.typography.font_size_base = size;
            save_constraints_to_file(&constraints);
            println!("🎨 Base font size updated: {}px", size);
        }
    });

    // Layout general settings using custom color pickers
    general_grid.attach(&window_color_picker, 0, 0, 2, 1);
    general_grid.attach(&primary_color_picker, 2, 0, 2, 1);
    general_grid.attach(&text_color_picker, 0, 1, 2, 1);

    let button_height_label = Label::new(Some("Button Height:"));
    button_height_label.set_halign(gtk4::Align::Start);
    general_grid.attach(&button_height_label, 2, 1, 1, 1);
    general_grid.attach(button_height_md_entry.widget(), 3, 1, 1, 1);

    let spacing_label = Label::new(Some("Base Spacing:"));
    spacing_label.set_halign(gtk4::Align::Start);
    general_grid.attach(&spacing_label, 0, 2, 1, 1);
    general_grid.attach(space_md_entry.widget(), 1, 2, 1, 1);

    let font_family_label = Label::new(Some("Font Family:"));
    font_family_label.set_halign(gtk4::Align::Start);
    general_grid.attach(&font_family_label, 2, 2, 1, 1);
    general_grid.attach(font_family_entry.widget(), 3, 2, 1, 1);

    let font_size_label = Label::new(Some("Font Size:"));
    font_size_label.set_halign(gtk4::Align::Start);
    general_grid.attach(&font_size_label, 0, 3, 1, 1);
    general_grid.attach(base_font_size_entry.widget(), 1, 3, 1, 1);

    general_frame.set_child(Some(&general_grid));
    main_box.append(&general_frame);

    // Button Settings Section
    let button_frame = Frame::new(Some("Button Settings"));
    let button_grid = Grid::new();
    button_grid.set_margin_top(10);
    button_grid.set_margin_bottom(10);
    button_grid.set_margin_start(10);
    button_grid.set_margin_end(10);
    button_grid.set_row_spacing(10);
    button_grid.set_column_spacing(15);

    // Button-specific controls - using custom color pickers
    let secondary_color_picker = create_custom_color_picker(&constraints.colors.button_secondary_bg, "Secondary Button:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.button_secondary_bg = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let danger_color_picker = create_custom_color_picker(&constraints.colors.button_danger_bg, "Danger Button:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.button_danger_bg = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let button_hover_color_picker = create_custom_color_picker(&constraints.colors.button_primary_hover, "Button Hover:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.button_primary_hover = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let button_height_sm_entry = EditorInput::number(theme.clone());
    let button_height_lg_entry = EditorInput::number(theme.clone());
    let border_radius_md_entry = EditorInput::number(theme.clone());
    let button_padding_x_entry = EditorInput::number(theme.clone());
    let button_padding_y_entry = EditorInput::number(theme.clone());

    // Set initial button values
    button_height_sm_entry.set_text(&constraints.geometry.button_height_sm.to_string());
    button_height_lg_entry.set_text(&constraints.geometry.button_height_lg.to_string());
    border_radius_md_entry.set_text(&constraints.geometry.border_radius_md.to_string());
    button_padding_x_entry.set_text(&constraints.geometry.button_padding_x.to_string());
    button_padding_y_entry.set_text(&constraints.geometry.button_padding_y.to_string());

    // Connect instant save for button entries
    button_height_sm_entry.connect_changed(move |text| {
        if let Ok(height) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.button_height_sm = height;
            save_constraints_to_file(&constraints);
            println!("🎨 Small button height updated: {}px", height);
        }
    });

    button_height_lg_entry.connect_changed(move |text| {
        if let Ok(height) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.button_height_lg = height;
            save_constraints_to_file(&constraints);
            println!("🎨 Large button height updated: {}px", height);
        }
    });

    border_radius_md_entry.connect_changed(move |text| {
        if let Ok(radius) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.border_radius_md = radius;
            save_constraints_to_file(&constraints);
            println!("🎨 Border radius updated: {}px", radius);
        }
    });

    button_padding_x_entry.connect_changed(move |text| {
        if let Ok(padding) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.button_padding_x = padding;
            save_constraints_to_file(&constraints);
            println!("🎨 Button padding X updated: {}px", padding);
        }
    });

    button_padding_y_entry.connect_changed(move |text| {
        if let Ok(padding) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.button_padding_y = padding;
            save_constraints_to_file(&constraints);
            println!("🎨 Button padding Y updated: {}px", padding);
        }
    });

    // Layout button settings using custom color pickers
    button_grid.attach(&secondary_color_picker, 0, 0, 2, 1);
    button_grid.attach(&danger_color_picker, 2, 0, 2, 1);
    button_grid.attach(&button_hover_color_picker, 0, 1, 2, 1);

    let button_height_sm_label = Label::new(Some("Small Button Height:"));
    button_height_sm_label.set_halign(gtk4::Align::Start);
    button_grid.attach(&button_height_sm_label, 2, 1, 1, 1);
    button_grid.attach(button_height_sm_entry.widget(), 3, 1, 1, 1);

    let button_height_lg_label = Label::new(Some("Large Button Height:"));
    button_height_lg_label.set_halign(gtk4::Align::Start);
    button_grid.attach(&button_height_lg_label, 0, 2, 1, 1);
    button_grid.attach(button_height_lg_entry.widget(), 1, 2, 1, 1);

    let border_radius_label = Label::new(Some("Button Border Radius:"));
    border_radius_label.set_halign(gtk4::Align::Start);
    button_grid.attach(&border_radius_label, 2, 2, 1, 1);
    button_grid.attach(border_radius_md_entry.widget(), 3, 2, 1, 1);

    let button_padding_x_label = Label::new(Some("Button Padding X:"));
    button_padding_x_label.set_halign(gtk4::Align::Start);
    button_grid.attach(&button_padding_x_label, 0, 3, 1, 1);
    button_grid.attach(button_padding_x_entry.widget(), 1, 3, 1, 1);

    let button_padding_y_label = Label::new(Some("Button Padding Y:"));
    button_padding_y_label.set_halign(gtk4::Align::Start);
    button_grid.attach(&button_padding_y_label, 2, 3, 1, 1);
    button_grid.attach(button_padding_y_entry.widget(), 3, 3, 1, 1);

    button_frame.set_child(Some(&button_grid));
    main_box.append(&button_frame);

    // Panel Settings Section
    let panel_frame = Frame::new(Some("Panel Settings"));
    let panel_grid = Grid::new();
    panel_grid.set_margin_top(10);
    panel_grid.set_margin_bottom(10);
    panel_grid.set_margin_start(10);
    panel_grid.set_margin_end(10);
    panel_grid.set_row_spacing(10);
    panel_grid.set_column_spacing(15);

    // Panel-specific controls - using our custom color picker
    let panel_color_picker = create_custom_color_picker(&constraints.colors.panel_background, "Panel Background:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.panel_background = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let toolbar_color_picker = create_custom_color_picker(&constraints.colors.toolbar_background, "Toolbar Background:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.toolbar_background = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let sidebar_color_picker = create_custom_color_picker(&constraints.colors.sidebar_background, "Sidebar Background:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.sidebar_background = color;
            save_constraints_to_file(&constraints);
        }
    });
    let panel_padding_entry = EditorInput::number(theme.clone());
    let panel_header_height_entry = EditorInput::number(theme.clone());
    let toolbar_height_entry = EditorInput::number(theme.clone());

    // Custom color pickers handle their own initial values
    panel_padding_entry.set_text(&constraints.spacing.panel_padding.to_string());
    panel_header_height_entry.set_text(&constraints.geometry.panel_header_height.to_string());
    toolbar_height_entry.set_text(&constraints.geometry.toolbar_height.to_string());

    // Connect instant save for panel entries
    panel_padding_entry.connect_changed(move |text| {
        if let Ok(padding) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.spacing.panel_padding = padding;
            save_constraints_to_file(&constraints);
            println!("🎨 Panel padding updated: {}px", padding);
        }
    });

    panel_header_height_entry.connect_changed(move |text| {
        if let Ok(height) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.panel_header_height = height;
            save_constraints_to_file(&constraints);
            println!("🎨 Panel header height updated: {}px", height);
        }
    });

    toolbar_height_entry.connect_changed(move |text| {
        if let Ok(height) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.toolbar_height = height;
            save_constraints_to_file(&constraints);
            println!("🎨 Toolbar height updated: {}px", height);
        }
    });

    // Layout panel settings using custom color pickers
    panel_grid.attach(&panel_color_picker, 0, 0, 2, 1);
    panel_grid.attach(&toolbar_color_picker, 2, 0, 2, 1);
    panel_grid.attach(&sidebar_color_picker, 0, 1, 2, 1);

    let panel_padding_label = Label::new(Some("Panel Padding:"));
    panel_padding_label.set_halign(gtk4::Align::Start);
    panel_grid.attach(&panel_padding_label, 2, 1, 1, 1);
    panel_grid.attach(panel_padding_entry.widget(), 3, 1, 1, 1);

    let panel_header_label = Label::new(Some("Panel Header Height:"));
    panel_header_label.set_halign(gtk4::Align::Start);
    panel_grid.attach(&panel_header_label, 0, 2, 1, 1);
    panel_grid.attach(panel_header_height_entry.widget(), 1, 2, 1, 1);

    let toolbar_height_label = Label::new(Some("Toolbar Height:"));
    toolbar_height_label.set_halign(gtk4::Align::Start);
    panel_grid.attach(&toolbar_height_label, 2, 2, 1, 1);
    panel_grid.attach(toolbar_height_entry.widget(), 3, 2, 1, 1);

    panel_frame.set_child(Some(&panel_grid));
    main_box.append(&panel_frame);

    // Input Settings Section
    let input_frame = Frame::new(Some("Input Settings"));
    let input_grid = Grid::new();
    input_grid.set_margin_top(10);
    input_grid.set_margin_bottom(10);
    input_grid.set_margin_start(10);
    input_grid.set_margin_end(10);
    input_grid.set_row_spacing(10);
    input_grid.set_column_spacing(15);

    // Input-specific controls - using custom color pickers
    let input_bg_color_picker = create_custom_color_picker(&constraints.colors.input_background, "Input Background:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.input_background = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let input_border_color_picker = create_custom_color_picker(&constraints.colors.input_border, "Input Border:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.input_border = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let text_secondary_color_picker = create_custom_color_picker(&constraints.colors.text_secondary, "Secondary Text:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.text_secondary = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let text_accent_color_picker = create_custom_color_picker(&constraints.colors.text_accent, "Accent Text:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.text_accent = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let input_height_entry = EditorInput::number(theme.clone());

    // Set initial input values
    input_height_entry.set_text(&constraints.geometry.input_height_sm.to_string());

    // Connect instant save for input height
    input_height_entry.connect_changed(move |text| {
        if let Ok(height) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.input_height_sm = height;
            constraints.geometry.input_height_md = height;
            constraints.geometry.input_height_lg = height;
            save_constraints_to_file(&constraints);
            println!("🎨 Input height updated: {}px", height);
        }
    });

    // Layout input settings using custom color pickers
    input_grid.attach(&input_bg_color_picker, 0, 0, 2, 1);
    input_grid.attach(&input_border_color_picker, 2, 0, 2, 1);
    input_grid.attach(&text_secondary_color_picker, 0, 1, 2, 1);
    input_grid.attach(&text_accent_color_picker, 2, 1, 2, 1);

    let input_height_label = Label::new(Some("Input Height:"));
    input_height_label.set_halign(gtk4::Align::Start);
    input_grid.attach(&input_height_label, 0, 2, 1, 1);
    input_grid.attach(input_height_entry.widget(), 1, 2, 1, 1);

    input_frame.set_child(Some(&input_grid));
    main_box.append(&input_frame);

    // Advanced Settings Section
    let advanced_frame = Frame::new(Some("Advanced Settings"));
    let advanced_grid = Grid::new();
    advanced_grid.set_margin_top(10);
    advanced_grid.set_margin_bottom(10);
    advanced_grid.set_margin_start(10);
    advanced_grid.set_margin_end(10);
    advanced_grid.set_row_spacing(10);
    advanced_grid.set_column_spacing(15);

    // Advanced spacing controls - using custom color picker
    let border_color_picker = create_custom_color_picker(&constraints.colors.border_primary, "Border Color:", {
        move |color| {
            let mut constraints = design_loader::load_current_design();
            constraints.colors.border_primary = color;
            save_constraints_to_file(&constraints);
        }
    });
    
    let space_xs_entry = EditorInput::number(theme.clone());
    let space_sm_entry = EditorInput::number(theme.clone());
    let space_lg_entry = EditorInput::number(theme.clone());
    let space_xl_entry = EditorInput::number(theme.clone());
    let border_radius_sm_entry = EditorInput::number(theme.clone());
    let border_radius_lg_entry = EditorInput::number(theme.clone());

    // Set initial advanced values and connect instant save
    space_xs_entry.set_text(&constraints.spacing.space_xs.to_string());
    space_sm_entry.set_text(&constraints.spacing.space_sm.to_string());
    space_lg_entry.set_text(&constraints.spacing.space_lg.to_string());
    space_xl_entry.set_text(&constraints.spacing.space_xl.to_string());
    border_radius_sm_entry.set_text(&constraints.geometry.border_radius_sm.to_string());
    border_radius_lg_entry.set_text(&constraints.geometry.border_radius_lg.to_string());

    // Connect instant save for all spacing entries
    space_xs_entry.connect_changed(move |text| {
        if let Ok(spacing) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.spacing.space_xs = spacing;
            save_constraints_to_file(&constraints);
            println!("🎨 XS spacing updated: {}px", spacing);
        }
    });

    space_lg_entry.connect_changed(move |text| {
        if let Ok(spacing) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.spacing.space_lg = spacing;
            save_constraints_to_file(&constraints);
            println!("🎨 LG spacing updated: {}px", spacing);
        }
    });

    space_sm_entry.connect_changed(move |text| {
        if let Ok(spacing) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.spacing.space_sm = spacing;
            save_constraints_to_file(&constraints);
            println!("🎨 SM spacing updated: {}px", spacing);
        }
    });

    space_xl_entry.connect_changed(move |text| {
        if let Ok(spacing) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.spacing.space_xl = spacing;
            save_constraints_to_file(&constraints);
            println!("🎨 XL spacing updated: {}px", spacing);
        }
    });

    border_radius_sm_entry.connect_changed(move |text| {
        if let Ok(radius) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.border_radius_sm = radius;
            save_constraints_to_file(&constraints);
            println!("🎨 Small border radius updated: {}px", radius);
        }
    });

    border_radius_lg_entry.connect_changed(move |text| {
        if let Ok(radius) = text.parse::<f32>() {
            let mut constraints = design_loader::load_current_design();
            constraints.geometry.border_radius_lg = radius;
            save_constraints_to_file(&constraints);
            println!("🎨 Large border radius updated: {}px", radius);
        }
    });

    // Layout advanced settings using custom color picker
    advanced_grid.attach(&border_color_picker, 0, 0, 2, 1);

    let space_xs_label = Label::new(Some("Extra Small Spacing:"));
    space_xs_label.set_halign(gtk4::Align::Start);
    advanced_grid.attach(&space_xs_label, 2, 0, 1, 1);
    advanced_grid.attach(space_xs_entry.widget(), 3, 0, 1, 1);

    let space_sm_label = Label::new(Some("Small Spacing:"));
    space_sm_label.set_halign(gtk4::Align::Start);
    advanced_grid.attach(&space_sm_label, 0, 1, 1, 1);
    advanced_grid.attach(space_sm_entry.widget(), 1, 1, 1, 1);

    let space_lg_label = Label::new(Some("Large Spacing:"));
    space_lg_label.set_halign(gtk4::Align::Start);
    advanced_grid.attach(&space_lg_label, 2, 1, 1, 1);
    advanced_grid.attach(space_lg_entry.widget(), 3, 1, 1, 1);

    let space_xl_label = Label::new(Some("Extra Large Spacing:"));
    space_xl_label.set_halign(gtk4::Align::Start);
    advanced_grid.attach(&space_xl_label, 0, 2, 1, 1);
    advanced_grid.attach(space_xl_entry.widget(), 1, 2, 1, 1);

    let border_radius_sm_label = Label::new(Some("Small Border Radius:"));
    border_radius_sm_label.set_halign(gtk4::Align::Start);
    advanced_grid.attach(&border_radius_sm_label, 2, 2, 1, 1);
    advanced_grid.attach(border_radius_sm_entry.widget(), 3, 2, 1, 1);

    let border_radius_lg_label = Label::new(Some("Large Border Radius:"));
    border_radius_lg_label.set_halign(gtk4::Align::Start);
    advanced_grid.attach(&border_radius_lg_label, 0, 3, 1, 1);
    advanced_grid.attach(border_radius_lg_entry.widget(), 1, 3, 1, 1);

    advanced_frame.set_child(Some(&advanced_grid));
    main_box.append(&advanced_frame);

    // Action buttons
    let button_box = Box::new(Orientation::Horizontal, 10);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(20);

    let save_button = EditorButton::primary("Save Settings", theme.clone());
    let reset_button = EditorButton::danger("Reset to Defaults", theme.clone());
    let test_button = EditorButton::secondary("Test in Editor", theme.clone());

    button_box.append(reset_button.widget());
    button_box.append(test_button.widget());
    button_box.append(save_button.widget());
    main_box.append(&button_box);

    window.set_child(Some(&main_box));

    // Custom color pickers handle their own state automatically
    
    // Connect save button (settings already auto-save via connect_changed)
    save_button.connect_clicked(move || {
        println!("💾 All settings are automatically saved when changed!");
        println!("✅ Current configuration is active.");
    });

    // Connect reset button
    reset_button.connect_clicked(move || {
        let defaults = DesignConstraints::unity_dark();
        
        // Reset to defaults and save
        save_constraints_to_file(&defaults);
        
        println!("🔄 Reset to defaults applied!");
        println!("🔄 Please restart the settings app to see the reset values.");
    });

    // Connect test button  
    test_button.connect_clicked(move || {
        // Colors are already saved by custom pickers, so just launch editor
        launch_editor();
    });

    window.present();
}

// This function is no longer needed - we use connect_color_set directly for instant updates

fn launch_editor() {
    use std::process::Command;
    
    println!("🚀 Launching Unity editor with current settings...");
    
    match Command::new("cargo")
        .args(&["run", "--bin", "unity-editor"])
        .spawn()
    {
        Ok(_) => {
            println!("✅ Unity editor launched successfully!");
        },
        Err(e) => {
            eprintln!("❌ Failed to launch Unity editor: {}", e);
        }
    }
}

fn save_comprehensive_settings(
    window_color_button: &ColorButton,
    primary_color_button: &ColorButton,
    text_color_button: &ColorButton,
    secondary_color_button: &ColorButton,
    danger_color_button: &ColorButton,
    panel_color_button: &ColorButton,
    toolbar_color_button: &ColorButton,
    sidebar_color_button: &ColorButton,
    input_bg_color_button: &ColorButton,
    input_border_color_button: &ColorButton,
    text_secondary_color_button: &ColorButton,
    text_accent_color_button: &ColorButton,
    border_color_button: &ColorButton,
    button_height_md_entry: &Entry,
    button_height_sm_entry: &Entry,
    button_height_lg_entry: &Entry,
    border_radius_md_entry: &Entry,
    border_radius_sm_entry: &Entry,
    border_radius_lg_entry: &Entry,
    space_md_entry: &Entry,
    space_xs_entry: &Entry,
    space_sm_entry: &Entry,
    space_lg_entry: &Entry,
    space_xl_entry: &Entry,
    font_family_entry: &Entry,
    base_font_size_entry: &Entry,
    panel_padding_entry: &Entry,
    panel_header_height_entry: &Entry,
    toolbar_height_entry: &Entry,
    input_height_entry: &Entry,
) {
    let mut constraints = DesignConstraints::unity_dark();

    // Update ALL colors
    constraints.colors.window_background = window_color_button.rgba().to_string();
    constraints.colors.button_primary_bg = primary_color_button.rgba().to_string();
    constraints.colors.text_primary = text_color_button.rgba().to_string();
    constraints.colors.button_secondary_bg = secondary_color_button.rgba().to_string();
    constraints.colors.button_danger_bg = danger_color_button.rgba().to_string();
    constraints.colors.panel_background = panel_color_button.rgba().to_string();
    constraints.colors.toolbar_background = toolbar_color_button.rgba().to_string();
    constraints.colors.sidebar_background = sidebar_color_button.rgba().to_string();
    constraints.colors.input_background = input_bg_color_button.rgba().to_string();
    constraints.colors.input_border = input_border_color_button.rgba().to_string();
    constraints.colors.text_secondary = text_secondary_color_button.rgba().to_string();
    constraints.colors.text_accent = text_accent_color_button.rgba().to_string();
    constraints.colors.border_primary = border_color_button.rgba().to_string();

    // Update ALL geometry
    if let Ok(height) = button_height_md_entry.text().parse::<f32>() {
        constraints.geometry.button_height_md = height;
    }
    if let Ok(height) = button_height_sm_entry.text().parse::<f32>() {
        constraints.geometry.button_height_sm = height;
    }
    if let Ok(height) = button_height_lg_entry.text().parse::<f32>() {
        constraints.geometry.button_height_lg = height;
    }
    if let Ok(radius) = border_radius_md_entry.text().parse::<f32>() {
        constraints.geometry.border_radius_md = radius;
    }
    if let Ok(radius) = border_radius_sm_entry.text().parse::<f32>() {
        constraints.geometry.border_radius_sm = radius;
    }
    if let Ok(radius) = border_radius_lg_entry.text().parse::<f32>() {
        constraints.geometry.border_radius_lg = radius;
    }
    if let Ok(height) = panel_header_height_entry.text().parse::<f32>() {
        constraints.geometry.panel_header_height = height;
    }
    if let Ok(height) = toolbar_height_entry.text().parse::<f32>() {
        constraints.geometry.toolbar_height = height;
    }
    if let Ok(height) = input_height_entry.text().parse::<f32>() {
        constraints.geometry.input_height_sm = height;
        constraints.geometry.input_height_md = height;
        constraints.geometry.input_height_lg = height;
    }

    // Update ALL spacing
    if let Ok(spacing) = space_md_entry.text().parse::<f32>() {
        constraints.spacing.space_md = spacing;
    }
    if let Ok(spacing) = space_xs_entry.text().parse::<f32>() {
        constraints.spacing.space_xs = spacing;
    }
    if let Ok(spacing) = space_sm_entry.text().parse::<f32>() {
        constraints.spacing.space_sm = spacing;
    }
    if let Ok(spacing) = space_lg_entry.text().parse::<f32>() {
        constraints.spacing.space_lg = spacing;
    }
    if let Ok(spacing) = space_xl_entry.text().parse::<f32>() {
        constraints.spacing.space_xl = spacing;
    }
    if let Ok(padding) = panel_padding_entry.text().parse::<f32>() {
        constraints.spacing.panel_padding = padding;
    }

    // Update typography
    constraints.typography.primary_font = font_family_entry.text().to_string();
    if let Ok(size) = base_font_size_entry.text().parse::<f32>() {
        constraints.typography.font_size_base = size;
    }

    // Save constraints
    if let Err(e) = std::fs::write("target/current_design_constraints.json", serde_json::to_string_pretty(&constraints).unwrap()) {
        eprintln!("❌ Failed to save settings: {}", e);
    } else {
        println!("🔥 Hot reload: Settings saved and applied!");
    }
}

// Keep the basic version for backwards compatibility
fn save_basic_settings(
    window_color_button: &ColorButton,
    primary_color_button: &ColorButton,
    text_color_button: &ColorButton,
    button_height_entry: &Entry,
    border_radius_entry: &Entry,
    spacing_entry: &Entry,
    font_family_entry: &Entry,
    base_font_size_entry: &Entry,
) {
    let mut constraints = DesignConstraints::unity_dark();

    // Update colors
    constraints.colors.window_background = window_color_button.rgba().to_string();
    constraints.colors.button_primary_bg = primary_color_button.rgba().to_string();
    constraints.colors.text_primary = text_color_button.rgba().to_string();

    // Update geometry
    if let Ok(height) = button_height_entry.text().parse::<f32>() {
        constraints.geometry.button_height_md = height;
    }
    if let Ok(radius) = border_radius_entry.text().parse::<f32>() {
        constraints.geometry.border_radius_md = radius;
    }

    // Update spacing
    if let Ok(spacing) = spacing_entry.text().parse::<f32>() {
        constraints.spacing.space_md = spacing;
    }

    // Update typography
    constraints.typography.primary_font = font_family_entry.text().to_string();
    if let Ok(size) = base_font_size_entry.text().parse::<f32>() {
        constraints.typography.font_size_base = size;
    }

    // Save constraints
    if let Err(e) = std::fs::write("target/current_design_constraints.json", serde_json::to_string_pretty(&constraints).unwrap()) {
        eprintln!("Failed to save settings: {}", e);
    } else {
        println!("✅ Settings saved successfully!");
    }
}
