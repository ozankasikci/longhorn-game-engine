use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Orientation, Paned, Label, TreeView, TreeStore, TreeViewColumn, CellRendererText, ScrolledWindow, TextView, CheckButton, SpinButton, Adjustment, Notebook, Frame, Separator, ComboBoxText, Scale};
use glib::{Type, ToValue};
use gio::{Menu, SimpleAction};
use libadwaita::prelude::*;
use libadwaita::{Application as AdwApplication, ApplicationWindow as AdwApplicationWindow, HeaderBar as AdwHeaderBar};

use engine_editor::ecs_editor_state::{EcsEditorState, ConsoleMessageType};
// use engine_core::Entity;
use engine_ui_system::{Vector3Field, EditorTheme, EditorButton, ButtonVariant, ButtonSize, EditorInput, AssetField};
use engine_ui_system::widgets::vector_field::Vector3;
use std::rc::Rc;
use std::cell::RefCell;

// Define possible drop zones in the main window
#[derive(Debug, Clone, PartialEq)]
enum DropZone {
    LeftSidebar,    // Hierarchy area
    RightSidebar,   // Inspector area  
    CenterTop,      // Scene/Game view area
    BottomTabs,     // Console/Project/etc area
    MainCenter,     // General center area
}
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::path::Path;
use notify::{Watcher, RecursiveMode, Event, EventKind, recommended_watcher};

const APP_ID: &str = "org.gameengine.UnityEditor";

#[derive(Debug)]
struct DetachablePanel {
    title: String,
    content: GtkBox,
    is_detached: bool,
    floating_window: Option<AdwApplicationWindow>,
    original_parent: Option<gtk4::Widget>,
}

impl DetachablePanel {
    fn new(title: String, content: GtkBox) -> Self {
        Self {
            title,
            content,
            is_detached: false,
            floating_window: None,
            original_parent: None,
        }
    }
}

struct UnityEditorApp {
    app: AdwApplication,
    editor_state: Rc<RefCell<EcsEditorState>>,
    toolbar_window: Option<AdwApplicationWindow>,
    detachable_panels: Rc<RefCell<std::collections::HashMap<String, DetachablePanel>>>,
}

impl UnityEditorApp {
    fn new() -> Self {
        let app = AdwApplication::builder()
            .application_id(APP_ID)
            .build();

        let editor_state = Rc::new(RefCell::new(EcsEditorState::new()));
        let detachable_panels = Rc::new(RefCell::new(std::collections::HashMap::new()));

        Self {
            app,
            editor_state,
            toolbar_window: None,
            detachable_panels,
        }
    }

    fn setup_actions(&self) {
        // File menu actions
        let new_action = SimpleAction::new("new", None);
        let open_action = SimpleAction::new("open", None);
        let save_action = SimpleAction::new("save", None);
        let quit_action = SimpleAction::new("quit", None);

        // GameObject menu actions
        let create_empty_action = SimpleAction::new("create-empty", None);
        let create_cube_action = SimpleAction::new("create-cube", None);
        let create_sphere_action = SimpleAction::new("create-sphere", None);

        // Window menu actions
        let show_hierarchy_action = SimpleAction::new("show-hierarchy", None);
        let show_inspector_action = SimpleAction::new("show-inspector", None);
        let show_console_action = SimpleAction::new("show-console", None);

        // Connect actions to callbacks
        let editor_state = self.editor_state.clone();
        create_empty_action.connect_activate(move |_, _| {
            let mut state = editor_state.borrow_mut();
            let entity = state.create_empty_object("Empty GameObject".to_string());
            println!("➕ Created Empty GameObject with Entity ID: {}", entity.id());
        });

        let editor_state = self.editor_state.clone();
        create_cube_action.connect_activate(move |_, _| {
            let mut state = editor_state.borrow_mut();
            let entity = state.create_cube("Cube".to_string());
            println!("🧊 Created Cube with Entity ID: {}", entity.id());
        });

        let editor_state = self.editor_state.clone();
        create_sphere_action.connect_activate(move |_, _| {
            let mut state = editor_state.borrow_mut();
            let entity = state.create_sphere("Sphere".to_string());
            println!("⚫ Created Sphere with Entity ID: {}", entity.id());
        });

        quit_action.connect_activate(glib::clone!(@weak self.app as app => move |_, _| {
            app.quit();
        }));

        // Add actions to application
        self.app.add_action(&new_action);
        self.app.add_action(&open_action);
        self.app.add_action(&save_action);
        self.app.add_action(&quit_action);
        self.app.add_action(&create_empty_action);
        self.app.add_action(&create_cube_action);
        self.app.add_action(&create_sphere_action);
        self.app.add_action(&show_hierarchy_action);
        self.app.add_action(&show_inspector_action);
        self.app.add_action(&show_console_action);
    }

    fn create_menu_model() -> Menu {
        let menu = Menu::new();

        // File menu
        let file_menu = Menu::new();
        file_menu.append(Some("New"), Some("app.new"));
        file_menu.append(Some("Open"), Some("app.open"));
        file_menu.append(Some("Save"), Some("app.save"));
        file_menu.append(Some("Quit"), Some("app.quit"));

        // GameObject menu
        let gameobject_menu = Menu::new();
        gameobject_menu.append(Some("Create Empty"), Some("app.create-empty"));
        gameobject_menu.append(Some("3D Object"), None);
        let object_3d_menu = Menu::new();
        object_3d_menu.append(Some("Cube"), Some("app.create-cube"));
        object_3d_menu.append(Some("Sphere"), Some("app.create-sphere"));
        gameobject_menu.append_submenu(Some("3D Object"), &object_3d_menu);

        // Window menu
        let window_menu = Menu::new();
        window_menu.append(Some("Hierarchy"), Some("app.show-hierarchy"));
        window_menu.append(Some("Inspector"), Some("app.show-inspector"));
        window_menu.append(Some("Console"), Some("app.show-console"));

        // Add sections to main menu
        menu.append_submenu(Some("File"), &file_menu);
        menu.append_submenu(Some("GameObject"), &gameobject_menu);
        menu.append_submenu(Some("Window"), &window_menu);

        menu
    }

    fn setup_hot_reload(&self, window: &AdwApplicationWindow) {
        // Create an atomic flag for signaling reloads
        let reload_flag = Arc::new(AtomicBool::new(false));
        let reload_flag_clone = reload_flag.clone();
        
        // Create a channel for file system events
        let (tx, rx) = mpsc::channel();
        
        // Set up file watcher in a separate thread
        thread::spawn(move || {
            let mut watcher = match recommended_watcher(tx) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("❌ Failed to create file watcher: {}", e);
                    return;
                }
            };

            let config_path = Path::new("target/current_design_constraints.json");
            if let Err(e) = watcher.watch(config_path, RecursiveMode::NonRecursive) {
                eprintln!("❌ Failed to watch design constraints file: {}", e);
                return;
            }

            println!("🔥 Hot reload enabled - watching for design constraint changes...");

            loop {
                match rx.recv() {
                    Ok(Ok(Event { kind: EventKind::Modify(_), .. })) | 
                    Ok(Ok(Event { kind: EventKind::Create(_), .. })) => {
                        println!("🔄 Design constraints file changed - signaling reload...");
                        reload_flag.store(true, Ordering::Relaxed);
                    },
                    Ok(Ok(_)) => {}, // Ignore other events
                    Ok(Err(e)) => {
                        eprintln!("❌ File watcher error: {}", e);
                        break;
                    },
                    Err(e) => {
                        eprintln!("❌ File watcher error: {}", e);
                        break;
                    }
                }
            }
        });

        // Set up periodic check for reload flag in the main thread (60fps)
        let window_weak = glib::object::ObjectExt::downgrade(window);
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            if reload_flag_clone.load(Ordering::Relaxed) {
                reload_flag_clone.store(false, Ordering::Relaxed);
                if let Some(window) = window_weak.upgrade() {
                    Self::reload_styles(&window);
                }
            }
            glib::ControlFlow::Continue
        });
    }

    fn reload_styles(window: &AdwApplicationWindow) {
        println!("🎨 Reloading design constraints and applying new styles...");
        
        // Reload design constraints
        let _constraints = engine_ui_system::apply_constraints_to_window(window);
        engine_ui_system::preview_current_constraints();
        
        println!("✅ Hot reload complete - new styles applied!");
    }

    // Panel detaching functionality
    fn detach_panel(&self, panel_id: &str, main_window: &AdwApplicationWindow) {
        let mut panels = self.detachable_panels.borrow_mut();
        if let Some(panel) = panels.get_mut(panel_id) {
            if panel.is_detached {
                return; // Already detached
            }

            // Store original parent for reattaching
            panel.original_parent = panel.content.parent();

            // Remove from current parent
            if let Some(parent) = panel.content.parent() {
                if let Ok(box_parent) = parent.clone().downcast::<GtkBox>() {
                    box_parent.remove(&panel.content);
                } else if let Ok(paned_parent) = parent.downcast::<Paned>() {
                    // Handle paned parents
                    paned_parent.set_start_child(gtk4::Widget::NONE);
                    paned_parent.set_end_child(gtk4::Widget::NONE);
                }
            }

            // Create floating window
            let floating_window = AdwApplicationWindow::builder()
                .application(&self.app)
                .title(&format!("{} - Detached", panel.title))
                .default_width(400)
                .default_height(500)
                .build();

            floating_window.set_content(Some(&panel.content));
            floating_window.present();

            panel.floating_window = Some(floating_window.clone());
            panel.is_detached = true;

            println!("🪟 Detached panel: {}", panel.title);
        }
    }

    fn reattach_panel(&self, panel_id: &str) {
        let mut panels = self.detachable_panels.borrow_mut();
        if let Some(panel) = panels.get_mut(panel_id) {
            if !panel.is_detached {
                return; // Not detached
            }

            // Close floating window
            if let Some(window) = &panel.floating_window {
                window.close();
            }

            // Reattach to original parent
            if let Some(parent) = &panel.original_parent {
                if let Ok(box_parent) = parent.clone().downcast::<GtkBox>() {
                    box_parent.append(&panel.content);
                }
            }

            panel.floating_window = None;
            panel.original_parent = None;
            panel.is_detached = false;

            println!("🔗 Reattached panel: {}", panel.title);
        }
    }

    // Static method to handle panel detaching from button callbacks
    fn handle_detach_panel(
        detachable_panels: &Rc<RefCell<std::collections::HashMap<String, DetachablePanel>>>,
        panel_id: &str,
        app: &AdwApplication,
        _main_window: &AdwApplicationWindow,
    ) {
        let mut panels = detachable_panels.borrow_mut();
        if let Some(panel) = panels.get_mut(panel_id) {
            if panel.is_detached {
                println!("⚠️ Panel {} is already detached", panel.title);
                return;
            }

            // Store original parent for reattaching
            panel.original_parent = panel.content.parent();

            // Remove from current parent
            if let Some(parent) = panel.content.parent() {
                if let Ok(box_parent) = parent.clone().downcast::<GtkBox>() {
                    box_parent.remove(&panel.content);
                } else if let Ok(paned_parent) = parent.downcast::<Paned>() {
                    // For paned widgets, we need to handle this more carefully
                    // We'll create a placeholder widget
                    let placeholder = GtkBox::new(Orientation::Vertical, 0);
                    placeholder.add_css_class("panel-placeholder");
                    
                    // Replace the panel with placeholder
                    if Some(&panel.content.clone().upcast()) == paned_parent.start_child().as_ref() {
                        paned_parent.set_start_child(Some(&placeholder));
                    } else if Some(&panel.content.clone().upcast()) == paned_parent.end_child().as_ref() {
                        paned_parent.set_end_child(Some(&placeholder));
                    }
                }
            }

            // Create floating window
            let floating_window = AdwApplicationWindow::builder()
                .application(app)
                .title(&format!("{} - Detached", panel.title))
                .default_width(400)
                .default_height(500)
                .build();

            // Apply the same styling to the floating window
            let _constraints = engine_ui_system::apply_constraints_to_window(&floating_window);

            // Create container for floating window with reattach button
            let floating_container = GtkBox::new(Orientation::Vertical, 0);
            
            // Create header with reattach button and drag capability
            let floating_header = GtkBox::new(Orientation::Horizontal, 0);
            floating_header.add_css_class("panel-header");
            floating_header.add_css_class("draggable-header");
            
            let title_label = Label::new(Some(&panel.title));
            title_label.set_markup(&format!("<b>{} - Detached</b>", panel.title));
            title_label.set_halign(gtk4::Align::Start);
            title_label.set_hexpand(true);
            title_label.add_css_class("panel-title");
            
            // Make header draggable
            Self::setup_panel_drag(&floating_header, panel_id, &panel.title, detachable_panels, app);
            
            let reattach_btn = gtk4::Button::with_label("⮌");
            reattach_btn.set_tooltip_text(Some("Reattach Panel"));
            reattach_btn.add_css_class("panel-action-btn");
            
            floating_header.append(&title_label);
            floating_header.append(&reattach_btn);
            
            // Add separator
            let separator = Separator::new(Orientation::Horizontal);
            separator.add_css_class("panel-separator");
            
            floating_container.append(&floating_header);
            floating_container.append(&separator);
            floating_container.append(&panel.content);

            floating_window.set_content(Some(&floating_container));
            
            // Connect reattach button
            let detachable_panels_clone = detachable_panels.clone();
            let panel_id_clone = panel_id.to_string();
            let window_weak = glib::object::ObjectExt::downgrade(&floating_window);
            
            reattach_btn.connect_clicked(move |_| {
                println!("🔗 Reattach button clicked for: {}", panel_id_clone);
                Self::handle_reattach_panel(&detachable_panels_clone, &panel_id_clone);
                if let Some(window) = window_weak.upgrade() {
                    window.close();
                }
            });
            
            // Also handle window close event for reattaching
            let detachable_panels_clone2 = detachable_panels.clone();
            let panel_id_clone2 = panel_id.to_string();
            
            floating_window.connect_close_request(move |_| {
                println!("🔗 Floating window closed, reattaching: {}", panel_id_clone2);
                Self::handle_reattach_panel(&detachable_panels_clone2, &panel_id_clone2);
                glib::Propagation::Proceed
            });

            floating_window.present();

            panel.floating_window = Some(floating_window.clone());
            panel.is_detached = true;

            println!("🪟 Successfully detached panel: {}", panel.title);
        } else {
            println!("⚠️ Panel not found: {}", panel_id);
        }
    }

    // Static method to handle panel reattaching from floating windows
    fn handle_reattach_panel(
        detachable_panels: &Rc<RefCell<std::collections::HashMap<String, DetachablePanel>>>,
        panel_id: &str,
    ) {
        let mut panels = detachable_panels.borrow_mut();
        if let Some(panel) = panels.get_mut(panel_id) {
            if !panel.is_detached {
                println!("⚠️ Panel {} is not detached", panel.title);
                return;
            }

            // Remove panel content from floating window container
            if let Some(parent) = panel.content.parent() {
                if let Ok(box_parent) = parent.downcast::<GtkBox>() {
                    box_parent.remove(&panel.content);
                }
            }

            // Reattach to original parent
            if let Some(original_parent) = &panel.original_parent {
                if let Ok(box_parent) = original_parent.clone().downcast::<GtkBox>() {
                    box_parent.append(&panel.content);
                } else if let Ok(paned_parent) = original_parent.clone().downcast::<Paned>() {
                    // Remove placeholder and restore panel
                    if let Some(start_child) = paned_parent.start_child() {
                        if start_child.has_css_class("panel-placeholder") {
                            paned_parent.set_start_child(Some(&panel.content));
                        }
                    }
                    if let Some(end_child) = paned_parent.end_child() {
                        if end_child.has_css_class("panel-placeholder") {
                            paned_parent.set_end_child(Some(&panel.content));
                        }
                    }
                }
            }

            // Close floating window if it exists
            if let Some(window) = &panel.floating_window {
                // Window will be closed by the caller
            }

            panel.floating_window = None;
            panel.original_parent = None;
            panel.is_detached = false;

            println!("🔗 Successfully reattached panel: {}", panel.title);
        } else {
            println!("⚠️ Panel not found for reattaching: {}", panel_id);
        }
    }

    // Setup custom drag behavior for panel headers that follows cursor
    fn setup_panel_drag(
        header: &GtkBox, 
        panel_id: &str, 
        title: &str,
        detachable_panels: &Rc<RefCell<std::collections::HashMap<String, DetachablePanel>>>,
        app: &AdwApplication,
    ) {
        // Create proper cursor
        if let Some(cursor) = gtk4::gdk::Cursor::from_name("grab", None) {
            header.set_cursor(Some(&cursor));
        }
        
        // Add CSS class for styling
        header.add_css_class("draggable-header");
        
        // Use GestureDrag for drag detection
        let gesture_drag = gtk4::GestureDrag::new();
        
        let panel_id_clone = panel_id.to_string();
        let title_clone = title.to_string();
        let detachable_panels_clone = detachable_panels.clone();
        let app_clone = app.clone();
        
        // Store references we need during drag
        let drag_context: Rc<RefCell<Option<DragPreview>>> = Rc::new(RefCell::new(None));
        
        struct DragPreview {
            window: AdwApplicationWindow,
            start_x: f64,
            start_y: f64,
        }
        
        // On drag begin - create a floating window that follows cursor
        let panel_id_clone2 = panel_id_clone.clone();
        let title_clone2 = title_clone.clone();
        let app_clone2 = app_clone.clone();
        let drag_context_clone = drag_context.clone();
        
        gesture_drag.connect_drag_begin(move |gesture, start_x, start_y| {
            println!("🚀 Drag begin for panel: {} at ({}, {})", panel_id_clone2, start_x, start_y);
            
            // Create a transparent floating preview window
            let preview_window = AdwApplicationWindow::new(&app_clone2);
            preview_window.set_title(Some(&format!("{} (Preview)", title_clone2)));
            preview_window.set_default_size(200, 150);
            preview_window.set_decorated(false);
            preview_window.set_resizable(false);
            preview_window.add_css_class("drag-preview-window");
            
            // Make window semi-transparent
            preview_window.set_opacity(0.7);
            
            // Create preview content
            let preview_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            preview_content.set_margin_start(12);
            preview_content.set_margin_end(12);
            preview_content.set_margin_top(12);
            preview_content.set_margin_bottom(12);
            
            let preview_label = gtk4::Label::new(Some(&format!("📱 {} (Dragging)", title_clone2)));
            preview_label.add_css_class("preview-label");
            preview_content.append(&preview_label);
            
            let hint_label = gtk4::Label::new(Some("Drop to dock elsewhere"));
            hint_label.add_css_class("preview-hint");
            preview_content.append(&hint_label);
            
            preview_window.set_content(Some(&preview_content));
            
            // Show preview window
            if let Some(_root) = gesture.widget().root() {
                // Position preview window and show it
                // Note: Window positioning might not work perfectly on Wayland due to security restrictions
                preview_window.present();
                
                // Store the preview context
                *drag_context_clone.borrow_mut() = Some(DragPreview {
                    window: preview_window,
                    start_x,
                    start_y,
                });
                
                println!("🪟 Created floating preview window");
            }
        });
        
        // On drag update - move the preview window to follow cursor
        let drag_context_clone2 = drag_context.clone();
        gesture_drag.connect_drag_update(move |_gesture, offset_x, offset_y| {
            if let Some(_preview) = drag_context_clone2.borrow().as_ref() {
                // Update preview window position to follow cursor
                // Note: Window positioning might not work on Wayland due to security restrictions
                println!("🎯 Drag update: offset ({:.0}, {:.0})", offset_x, offset_y);
            }
        });
        
        // On drag end - handle drop or cancel
        let panel_id_clone3 = panel_id_clone.clone();
        let detachable_panels_clone3 = detachable_panels.clone();
        let app_clone3 = app_clone.clone();
        let drag_context_clone3 = drag_context.clone();
        
        gesture_drag.connect_drag_end(move |gesture, offset_x, offset_y| {
            println!("🏁 Drag ended for panel: {} at offset ({}, {})", panel_id_clone3, offset_x, offset_y);
            
            // Get start position from stored context
            let start_position = if let Some(preview) = drag_context_clone3.borrow().as_ref() {
                (preview.start_x, preview.start_y)
            } else {
                (0.0, 0.0)
            };
            
            // Clean up preview window
            if let Some(preview) = drag_context_clone3.borrow_mut().take() {
                preview.window.close();
                println!("🗑️ Closed preview window");
            }
            
            // Check if drag was significant enough for repositioning
            let drag_distance = (offset_x * offset_x + offset_y * offset_y).sqrt();
            if drag_distance > 50.0 {
                println!("🎯 Significant drag ({:.1}px) - checking drop zone", drag_distance);
                
                // Calculate final drop position
                let final_x = start_position.0 + offset_x;
                let final_y = start_position.1 + offset_y;
                
                // Get the drop location and check for valid drop zones
                if let Some(drop_zone) = Self::detect_drop_zone(gesture, final_x, final_y) {
                    println!("📍 Valid drop zone detected: {:?}", drop_zone);
                    
                    // Move panel to the detected drop zone
                    if let Some(main_window) = app_clone3.active_window() {
                        if let Ok(adw_window) = main_window.downcast::<AdwApplicationWindow>() {
                            Self::dock_panel_to_zone(&detachable_panels_clone3, &panel_id_clone3, &drop_zone, &adw_window);
                            println!("✅ Panel {} docked to {:?}", panel_id_clone3, drop_zone);
                        }
                    }
                } else {
                    println!("🪟 No valid drop zone - detaching to floating window");
                    
                    // No valid drop zone, detach to floating window
                    if let Some(main_window) = app_clone3.active_window() {
                        if let Ok(adw_window) = main_window.downcast::<AdwApplicationWindow>() {
                            Self::handle_detach_panel(&detachable_panels_clone3, &panel_id_clone3, &app_clone3, &adw_window);
                            println!("✅ Panel {} detached to floating window", panel_id_clone3);
                        }
                    }
                }
            } else {
                println!("🔒 Drag distance too small ({:.1}px) - keeping panel attached", drag_distance);
            }
        });
        
        header.add_controller(gesture_drag);
    }

    // Detect which drop zone the user is hovering over
    fn detect_drop_zone(gesture: &gtk4::GestureDrag, final_x: f64, final_y: f64) -> Option<DropZone> {
        println!("🔍 Detecting drop zone at position ({:.0}, {:.0})", final_x, final_y);
        
        // Get the main window widget to check relative positions
        if let Some(root) = gesture.widget().root() {
            if let Ok(main_window) = root.downcast::<AdwApplicationWindow>() {
                // Get window dimensions
                let window_width = main_window.width() as f64;
                let window_height = main_window.height() as f64;
                
                println!("🖥️ Window dimensions: {}x{}", window_width, window_height);
                
                // Define drop zone boundaries (rough estimates based on layout)
                let left_sidebar_width = 250.0;  // Hierarchy panel width
                let right_sidebar_width = 300.0; // Inspector panel width  
                let bottom_height = 200.0;       // Bottom tabs height
                let toolbar_height = 80.0;       // Top toolbar height
                
                // Check which zone the cursor is in
                if final_x < left_sidebar_width {
                    println!("📍 Drop zone: Left Sidebar (Hierarchy area)");
                    return Some(DropZone::LeftSidebar);
                } else if final_x > window_width - right_sidebar_width {
                    println!("📍 Drop zone: Right Sidebar (Inspector area)");
                    return Some(DropZone::RightSidebar);
                } else if final_y > window_height - bottom_height {
                    println!("📍 Drop zone: Bottom Tabs (Console/Project area)");
                    return Some(DropZone::BottomTabs);
                } else if final_y > toolbar_height && final_y < window_height - bottom_height {
                    if final_x > left_sidebar_width && final_x < window_width - right_sidebar_width {
                        println!("📍 Drop zone: Center Top (Scene/Game area)");
                        return Some(DropZone::CenterTop);
                    } else {
                        println!("📍 Drop zone: Main Center");
                        return Some(DropZone::MainCenter);
                    }
                }
            }
        }
        
        println!("❌ No valid drop zone detected");
        None
    }

    // Dock panel to the specified drop zone
    fn dock_panel_to_zone(
        detachable_panels: &Rc<RefCell<std::collections::HashMap<String, DetachablePanel>>>,
        panel_id: &str,
        drop_zone: &DropZone,
        main_window: &AdwApplicationWindow,
    ) {
        println!("🎯 Docking panel '{}' to zone: {:?}", panel_id, drop_zone);
        
        // For now, implement basic zone-based docking
        // In a real implementation, you would need to:
        // 1. Remove panel from current location
        // 2. Add panel to target location
        // 3. Update layout and resize containers
        
        match drop_zone {
            DropZone::LeftSidebar => {
                println!("📌 Would dock panel to left sidebar (Hierarchy area)");
                // TODO: Implement actual docking to left paned area
            },
            DropZone::RightSidebar => {
                println!("📌 Would dock panel to right sidebar (Inspector area)");
                // TODO: Implement actual docking to right paned area
            },
            DropZone::CenterTop => {
                println!("📌 Would dock panel to center area (Scene/Game tabs)");
                // TODO: Implement actual docking to center tabbed area
            },
            DropZone::BottomTabs => {
                println!("📌 Would dock panel to bottom tabs (Console/Project area)");
                // TODO: Implement actual docking to bottom notebook
            },
            DropZone::MainCenter => {
                println!("📌 Would dock panel to main center area");
                // TODO: Implement actual docking to center area
            },
        }
        
        // For now, just print the intended action 
        // This gives us the drop zone detection working, and we can implement 
        // actual panel repositioning in the next step
        println!("✅ Drop zone detection successful! Panel would be docked to {:?}", drop_zone);
        println!("⚠️ Actual panel repositioning will be implemented next");
        
        // For demonstration, let's keep the panel in its current location for now
        // rather than detaching it to floating window
    }

    // Setup drop zones in main window  
    fn setup_drop_zones(main_window: &AdwApplicationWindow, detachable_panels: &Rc<RefCell<std::collections::HashMap<String, DetachablePanel>>>) {
        // Accept text/plain content type
        let drop_target = gtk4::DropTarget::builder()
            .actions(gtk4::gdk::DragAction::MOVE)
            .formats(&gtk4::gdk::ContentFormats::for_type(glib::Type::STRING))
            .build();
        
        let detachable_panels_clone = detachable_panels.clone();
        drop_target.connect_drop(move |_, value, _, _| {
            println!("📍 Drop event received, attempting to get panel ID...");
            
            // Try to get the panel ID from the dropped data
            if let Ok(panel_id) = value.get::<String>() {
                println!("📍 Panel dropped in main window: {}", panel_id);
                
                // For now, just reattach to original position
                // TODO: Implement position-based docking
                Self::handle_reattach_panel(&detachable_panels_clone, &panel_id);
                return true;
            } else {
                println!("⚠️ Failed to get panel ID from dropped data");
            }
            false
        });
        
        drop_target.connect_enter(|_, _, _| {
            println!("🎯 Panel entered drop zone");
            gtk4::gdk::DragAction::MOVE
        });
        
        drop_target.connect_leave(|_| {
            println!("🚫 Panel left drop zone");
        });
        
        main_window.add_controller(drop_target);
        println!("✅ Drop zones set up on main window");
    }

    // Unified component creation methods
    fn create_standard_separator(&self) -> Separator {
        let separator = Separator::new(Orientation::Horizontal);
        separator.add_css_class("panel-separator");
        separator
    }

    fn create_standard_header(&self, title: &str) -> GtkBox {
        self.create_standard_header_with_actions(title, true)
    }

    fn create_standard_header_with_actions(&self, title: &str, show_detach: bool) -> GtkBox {
        let header = GtkBox::new(Orientation::Horizontal, 0);
        header.add_css_class("panel-header");
        
        let title_label = Label::new(Some(title));
        title_label.set_markup(&format!("<b>{}</b>", title));
        title_label.set_halign(gtk4::Align::Start);
        title_label.set_hexpand(true);
        title_label.add_css_class("panel-title");
        
        header.append(&title_label);
        
        if show_detach {
            // Add panel actions (detach button)
            let actions_box = GtkBox::new(Orientation::Horizontal, 2);
            
            let button_theme = Rc::new(RefCell::new(EditorTheme::default()));
            let detach_btn = EditorButton::new("⧉", ButtonVariant::Ghost, ButtonSize::Small, button_theme);
            detach_btn.widget().set_tooltip_text(Some("Detach Panel"));
            detach_btn.widget().add_css_class("panel-action-btn");
            
            actions_box.append(detach_btn.widget());
            header.append(&actions_box);
        }
        
        header
    }
    
    fn create_tabbed_header(&self, default_tab: &str) -> (GtkBox, GtkBox) {
        let header = GtkBox::new(Orientation::Horizontal, 0);
        header.add_css_class("panel-header");
        
        // Tab container for buttons
        let tab_container = GtkBox::new(Orientation::Horizontal, 0);
        tab_container.add_css_class("tab-container");
        header.append(&tab_container);
        
        // Spacer to push content to the right if needed
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);
        
        (header, tab_container)
    }

    fn create_standard_panel(&self, title: &str) -> (GtkBox, GtkBox) {
        self.create_detachable_panel(title, &AdwApplicationWindow::builder().build())
    }

    fn create_detachable_panel(&self, title: &str, main_window: &AdwApplicationWindow) -> (GtkBox, GtkBox) {
        let panel = GtkBox::new(Orientation::Vertical, 0);
        panel.add_css_class("standard-panel");

        // Header with detach functionality
        let header = self.create_standard_header_with_actions(title, true);
        panel.append(&header);
        
        // Separator
        let separator = self.create_standard_separator();
        panel.append(&separator);

        // Register as detachable panel
        let panel_id = title.to_lowercase().replace(" ", "_");
        let detachable_panel = DetachablePanel::new(title.to_string(), panel.clone());
        self.detachable_panels.borrow_mut().insert(panel_id.clone(), detachable_panel);

        // Connect detach button with actual detaching functionality
        if let Some(detach_btn) = self.find_detach_button(&header) {
            let panel_id_clone = panel_id.clone();
            let title_clone = title.to_string();
            let detachable_panels = self.detachable_panels.clone();
            let app = self.app.clone();
            let main_window_weak = glib::object::ObjectExt::downgrade(main_window);
            
            detach_btn.connect_clicked(move |_| {
                println!("🪟 Detaching panel: {} ({})", title_clone, panel_id_clone);
                
                if let Some(main_window) = main_window_weak.upgrade() {
                    Self::handle_detach_panel(&detachable_panels, &panel_id_clone, &app, &main_window);
                }
            });
        }

        // Add drag behavior to the header for drag-to-detach functionality
        Self::setup_panel_drag(&header, &panel_id, title, &self.detachable_panels, &self.app);

        (panel, header)
    }

    fn find_detach_button(&self, header: &GtkBox) -> Option<gtk4::Button> {
        // Find the detach button in the header
        let mut child = header.first_child();
        while let Some(widget) = child {
            if let Ok(box_widget) = widget.clone().downcast::<GtkBox>() {
                let mut button_child = box_widget.first_child();
                while let Some(button_widget) = button_child {
                    if let Ok(button) = button_widget.clone().downcast::<gtk4::Button>() {
                        if button.has_css_class("panel-action-btn") {
                            return Some(button);
                        }
                    }
                    button_child = button_widget.next_sibling();
                }
            }
            child = widget.next_sibling();
        }
        None
    }

    fn create_standard_content_area(&self) -> GtkBox {
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.add_css_class("panel-content");
        content.set_hexpand(true);
        content.set_vexpand(true);
        content
    }

    fn create_hierarchy_panel(&self, main_window: &AdwApplicationWindow) -> GtkBox {
        // Create standard panel for consistency with other panels
        let (panel, _header) = self.create_detachable_panel("Hierarchy", main_window);
        let content = self.create_standard_content_area();
        panel.append(&content);
        
        // Tree view for hierarchy
        let tree_store = TreeStore::new(&[Type::STRING, Type::BOOL, Type::U32]);
        let tree_view = TreeView::with_model(&tree_store);
        tree_view.set_headers_visible(false);

        // Name column
        let name_column = TreeViewColumn::new();
        name_column.set_title("Name");
        let name_renderer = CellRendererText::new();
        name_column.pack_start(&name_renderer, true);
        name_column.add_attribute(&name_renderer, "text", 0);
        tree_view.append_column(&name_column);

        // Populate with default objects from ECS
        let editor_state = self.editor_state.borrow();
        for (entity, name) in editor_state.get_named_entities() {
            let iter = tree_store.append(None);
            tree_store.set_value(&iter, 0, &name.to_value());
            tree_store.set_value(&iter, 1, &true.to_value()); // Always active for ECS entities
            tree_store.set_value(&iter, 2, &entity.id().to_value());
        }

        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_child(Some(&tree_view));
        scrolled.set_vexpand(true);

        content.append(&scrolled);

        panel
    }

    fn create_inspector_panel(&self, main_window: &AdwApplicationWindow) -> GtkBox {
        let (panel, _header) = self.create_detachable_panel("Inspector", main_window);
        let content = self.create_standard_content_area();
        panel.append(&content);

        // Create a basic theme for all components (Vector3Field, EditorInput, EditorButton)
        // Note: In a full implementation, this would use the theme system
        let basic_theme = Rc::new(RefCell::new(EditorTheme::default()));

        // GameObject section
        let gameobject_box = GtkBox::new(Orientation::Vertical, 2);
        gameobject_box.set_margin_start(2);
        gameobject_box.set_margin_end(2);
        gameobject_box.set_margin_top(2);
        gameobject_box.set_margin_bottom(2);

        // Name field
        let name_box = GtkBox::new(Orientation::Horizontal, 2);
        let name_label = Label::new(Some("Name:"));
        name_label.set_halign(gtk4::Align::Start);
        name_label.set_size_request(80, -1);
        let name_input = EditorInput::text_input(basic_theme.clone());
        name_input.set_text("GameObject");
        name_input.widget().set_hexpand(true);
        name_box.append(&name_label);
        name_box.append(name_input.widget());

        // Active checkbox
        let active_check = CheckButton::with_label("Active");
        active_check.set_active(true);

        gameobject_box.append(&name_box);
        gameobject_box.append(&active_check);
        content.append(&gameobject_box);

        // Transform section using reusable Vector3Field components
        let transform_box = GtkBox::new(Orientation::Vertical, 2);
        transform_box.set_margin_start(2);
        transform_box.set_margin_end(2);
        transform_box.set_margin_top(2);
        transform_box.set_margin_bottom(2);

        // Position Vector3Field
        let mut position_field = Vector3Field::position(basic_theme.clone());
        position_field.set_value(Vector3::zero()); // Default position
        transform_box.append(position_field.widget());

        // Rotation Vector3Field  
        let mut rotation_field = Vector3Field::rotation(basic_theme.clone());
        rotation_field.set_value(Vector3::zero()); // Default rotation
        transform_box.append(rotation_field.widget());

        // Scale Vector3Field
        let mut scale_field = Vector3Field::scale(basic_theme.clone());
        scale_field.set_value(Vector3::one()); // Default scale (1,1,1)
        transform_box.append(scale_field.widget());

        content.append(&transform_box);

        // Add Component section
        let component_box = GtkBox::new(Orientation::Vertical, 2);
        component_box.set_margin_start(2);
        component_box.set_margin_end(2);
        component_box.set_margin_top(2);
        component_box.set_margin_bottom(2);

        // Mesh Renderer Component
        let mesh_renderer_box = GtkBox::new(Orientation::Vertical, 2);
        mesh_renderer_box.set_margin_start(2);
        mesh_renderer_box.set_margin_end(2);
        mesh_renderer_box.set_margin_top(2);
        mesh_renderer_box.set_margin_bottom(2);

        // Mesh field for selecting 3D model
        let mut mesh_field = AssetField::mesh("Mesh:", basic_theme.clone());
        mesh_field.set_asset(None); // No mesh selected initially
        mesh_renderer_box.append(mesh_field.widget());

        // Material field using unified AssetField component
        let mut material_field = AssetField::material("Materials:", basic_theme.clone());
        material_field.set_asset(None); // No material selected initially
        mesh_renderer_box.append(material_field.widget());
        component_box.append(&mesh_renderer_box);

        // Collider Component
        let collider_box = GtkBox::new(Orientation::Vertical, 2);
        collider_box.set_margin_start(2);
        collider_box.set_margin_end(2);
        collider_box.set_margin_top(2);
        collider_box.set_margin_bottom(2);

        // Is Trigger checkbox
        let trigger_check = CheckButton::with_label("Is Trigger");
        trigger_check.set_active(false);
        collider_box.append(&trigger_check);

        // Size fields
        let size_label = Label::new(Some("Size"));
        size_label.set_markup("<b>Size</b>");
        size_label.set_halign(gtk4::Align::Start);
        collider_box.append(&size_label);

        let size_box = GtkBox::new(Orientation::Horizontal, 2);
        for axis in ["X", "Y", "Z"] {
            let axis_box = GtkBox::new(Orientation::Horizontal, 2);
            let axis_label = Label::new(Some(axis));
            axis_label.set_size_request(15, -1);
            let adjustment = Adjustment::new(1.0, 0.0, 100.0, 0.1, 1.0, 0.0);
            let spin = SpinButton::new(Some(&adjustment), 0.1, 2);
            spin.set_hexpand(true);
            
            match axis {
                "X" => axis_label.add_css_class("error"),
                "Y" => axis_label.add_css_class("success"),
                "Z" => axis_label.add_css_class("accent"),
                _ => {}
            }
            
            axis_box.append(&axis_label);
            axis_box.append(&spin);
            size_box.append(&axis_box);
        }
        collider_box.append(&size_box);

        component_box.append(&collider_box);

        // Add Component button
        let add_comp_btn = EditorButton::primary("Add Component", basic_theme.clone());
        add_comp_btn.widget().set_margin_top(2);
        component_box.append(add_comp_btn.widget());

        content.append(&component_box);

        panel
    }

    fn create_scene_view(&self) -> GtkBox {
        let scene_box = GtkBox::new(Orientation::Vertical, 0);

        // Scene view header (simplified - main controls are now in floating toolbar)
        let scene_header = GtkBox::new(Orientation::Horizontal, 4);
        scene_header.set_margin_start(4);
        scene_header.set_margin_end(4);
        scene_header.set_margin_top(4);
        scene_header.set_margin_bottom(4);
        scene_header.add_css_class("scene-header");

        // Scene view title
        let scene_title = Label::new(Some("Scene"));
        scene_title.set_markup("<b>Scene</b>");
        scene_title.set_halign(gtk4::Align::Start);
        
        // Scene options (right side)
        let scene_options = GtkBox::new(Orientation::Horizontal, 2);
        
        // Create theme for scene buttons
        let scene_theme = Rc::new(RefCell::new(EditorTheme::default()));
        
        let lighting_btn = EditorButton::toolbar_icon("weather-clear", "Scene Lighting", scene_theme.clone());
        let audio_btn = EditorButton::toolbar_icon("audio-volume-high", "Scene Audio", scene_theme.clone());
        let effects_btn = EditorButton::toolbar_icon("applications-graphics", "Scene Effects", scene_theme.clone());

        scene_options.append(lighting_btn.widget());
        scene_options.append(audio_btn.widget());
        scene_options.append(effects_btn.widget());

        // Spacer
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        scene_header.append(&scene_title);
        scene_header.append(&spacer);
        scene_header.append(&scene_options);

        scene_box.append(&scene_header);

        // Scene view area
        let scene_area = gtk4::DrawingArea::new();
        scene_area.set_vexpand(true);
        scene_area.set_hexpand(true);

        // Custom draw function for grid
        scene_area.set_draw_func(|_, context, width, height| {
            // Dark background
            context.set_source_rgb(0.1, 0.1, 0.1);
            context.paint().unwrap();

            // Grid
            context.set_source_rgb(0.3, 0.3, 0.3);
            context.set_line_width(1.0);

            let grid_size = 20.0;
            
            // Vertical lines
            let mut x = 0.0;
            while x < width as f64 {
                context.move_to(x, 0.0);
                context.line_to(x, height as f64);
                context.stroke().unwrap();
                x += grid_size;
            }

            // Horizontal lines
            let mut y = 0.0;
            while y < height as f64 {
                context.move_to(0.0, y);
                context.line_to(width as f64, y);
                context.stroke().unwrap();
                y += grid_size;
            }

            // Center marker
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            context.set_source_rgb(1.0, 0.4, 0.4);
            context.arc(center_x, center_y, 5.0, 0.0, 2.0 * std::f64::consts::PI);
            context.fill().unwrap();

            // Scene info overlay
            context.set_source_rgba(0.0, 0.0, 0.0, 0.7);
            context.rectangle(10.0, 10.0, 120.0, 60.0);
            context.fill().unwrap();

            context.set_source_rgb(1.0, 1.0, 1.0);
            context.move_to(15.0, 30.0);
            context.show_text("Scene View").unwrap();
            context.move_to(15.0, 45.0);
            context.show_text("📹 Perspective").unwrap();
            context.move_to(15.0, 60.0);
            context.show_text("🎯 Camera").unwrap();
        });

        // Handle clicks
        let click_gesture = gtk4::GestureClick::new();
        click_gesture.connect_pressed(|_, _, x, y| {
            println!("🎮 Scene clicked at: ({:.1}, {:.1})", x, y);
        });
        scene_area.add_controller(click_gesture);

        scene_box.append(&scene_area);

        scene_box
    }

    fn create_game_view(&self) -> GtkBox {
        let game_box = GtkBox::new(Orientation::Vertical, 0);

        // Game toolbar
        let toolbar = GtkBox::new(Orientation::Horizontal, 2);
        toolbar.set_margin_start(2);
        toolbar.set_margin_end(2);
        toolbar.set_margin_top(2);
        toolbar.set_margin_bottom(2);
        toolbar.add_css_class("toolbar");

        // Resolution dropdown
        let resolution_label = Label::new(Some("Resolution:"));
        let resolution_combo = ComboBoxText::new();
        resolution_combo.append_text("1920x1080");
        resolution_combo.append_text("1280x720");
        resolution_combo.append_text("1024x768");
        resolution_combo.append_text("Free Aspect");
        resolution_combo.set_active(Some(0));

        // Stats toggle
        let stats_check = CheckButton::with_label("Stats");
        stats_check.set_active(true);

        // Gizmos toggle
        let gizmos_check = CheckButton::with_label("Gizmos");
        gizmos_check.set_active(false);

        toolbar.append(&resolution_label);
        toolbar.append(&resolution_combo);
        toolbar.append(&stats_check);
        toolbar.append(&gizmos_check);

        // Spacer
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);

        // Maximize button
        let game_theme = Rc::new(RefCell::new(EditorTheme::default()));
        let maximize_btn = EditorButton::toolbar_icon("view-fullscreen", "Maximize on Play", game_theme.clone());
        toolbar.append(maximize_btn.widget());

        game_box.append(&toolbar);

        // Game view area
        let game_area = gtk4::DrawingArea::new();
        game_area.set_vexpand(true);
        game_area.set_hexpand(true);

        // Custom draw function for game view
        game_area.set_draw_func(|_, context, width, height| {
            // Game background
            context.set_source_rgb(0.05, 0.05, 0.1);
            context.paint().unwrap();

            // Game border
            context.set_source_rgb(0.2, 0.4, 0.8);
            context.set_line_width(2.0);
            context.rectangle(0.0, 0.0, width as f64, height as f64);
            context.stroke().unwrap();

            // Center "game camera" view
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            
            // Camera frame
            context.set_source_rgb(0.3, 0.3, 0.3);
            context.rectangle(50.0, 50.0, (width - 100) as f64, (height - 100) as f64);
            context.stroke().unwrap();

            // Sample game objects
            context.set_source_rgb(0.8, 0.2, 0.2);
            context.arc(center_x - 50.0, center_y, 15.0, 0.0, 2.0 * std::f64::consts::PI);
            context.fill().unwrap();

            context.set_source_rgb(0.2, 0.8, 0.2);
            context.rectangle(center_x + 20.0, center_y - 20.0, 40.0, 40.0);
            context.fill().unwrap();

            context.set_source_rgb(0.2, 0.2, 0.8);
            context.arc(center_x + 80.0, center_y + 30.0, 20.0, 0.0, 2.0 * std::f64::consts::PI);
            context.fill().unwrap();

            // Game stats overlay
            context.set_source_rgba(0.0, 0.0, 0.0, 0.8);
            context.rectangle(10.0, 10.0, 150.0, 80.0);
            context.fill().unwrap();

            context.set_source_rgb(0.0, 1.0, 0.0);
            context.move_to(15.0, 30.0);
            context.show_text("🎮 Game View").unwrap();
            context.move_to(15.0, 45.0);
            context.show_text("FPS: 60").unwrap();
            context.move_to(15.0, 60.0);
            context.show_text("Batches: 12").unwrap();
            context.move_to(15.0, 75.0);
            context.show_text("Tris: 2.4K").unwrap();
        });

        // Handle clicks
        let click_gesture = gtk4::GestureClick::new();
        click_gesture.connect_pressed(|_, _, x, y| {
            println!("🎮 Game view clicked at: ({:.1}, {:.1})", x, y);
        });
        game_area.add_controller(click_gesture);

        game_box.append(&game_area);

        game_box
    }

    fn create_console_panel(&self, main_window: &AdwApplicationWindow) -> GtkBox {
        let (panel, _header) = self.create_detachable_panel("Console", main_window);
        let content = self.create_standard_content_area();
        panel.append(&content);

        // Console output
        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.add_css_class("monospace");

        let buffer = text_view.buffer();
        
        // Add initial messages
        let editor_state = self.editor_state.borrow();
        for msg in &editor_state.console_messages {
            let icon = match msg.message_type {
                ConsoleMessageType::Info => "ℹ️",
                ConsoleMessageType::Warning => "⚠️",
                ConsoleMessageType::Error => "❌",
            };
            let line = format!("{} {}\n", icon, msg.message);
            let mut end_iter = buffer.end_iter();
            buffer.insert(&mut end_iter, &line);
        }

        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_child(Some(&text_view));
        scrolled.set_vexpand(true);

        content.append(&scrolled);

        panel
    }

    fn create_project_panel(&self, main_window: &AdwApplicationWindow) -> GtkBox {
        let (panel, _header) = self.create_detachable_panel("Project", main_window);
        let content = self.create_standard_content_area();
        panel.append(&content);

        // Tree view for project files
        let tree_store = TreeStore::new(&[Type::STRING, Type::STRING]);
        let tree_view = TreeView::with_model(&tree_store);
        tree_view.set_headers_visible(false);

        // File column
        let file_column = TreeViewColumn::new();
        file_column.set_title("File");
        let file_renderer = CellRendererText::new();
        file_column.pack_start(&file_renderer, true);
        file_column.add_attribute(&file_renderer, "text", 0);
        tree_view.append_column(&file_column);

        // Add sample project structure
        let scripts_iter = tree_store.append(None);
        tree_store.set_value(&scripts_iter, 0, &"📁 Scripts".to_value());
        tree_store.set_value(&scripts_iter, 1, &"folder".to_value());

        let materials_iter = tree_store.append(None);
        tree_store.set_value(&materials_iter, 0, &"📁 Materials".to_value());
        tree_store.set_value(&materials_iter, 1, &"folder".to_value());

        let textures_iter = tree_store.append(None);
        tree_store.set_value(&textures_iter, 0, &"📁 Textures".to_value());
        tree_store.set_value(&textures_iter, 1, &"folder".to_value());

        let models_iter = tree_store.append(None);
        tree_store.set_value(&models_iter, 0, &"📁 Models".to_value());
        tree_store.set_value(&models_iter, 1, &"folder".to_value());

        let scenes_iter = tree_store.append(None);
        tree_store.set_value(&scenes_iter, 0, &"📁 Scenes".to_value());
        tree_store.set_value(&scenes_iter, 1, &"folder".to_value());

        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_child(Some(&tree_view));
        scrolled.set_vexpand(true);

        content.append(&scrolled);

        panel
    }

    fn create_animation_panel(&self) -> GtkBox {
        let panel = GtkBox::new(Orientation::Vertical, 2);
        panel.set_margin_start(2);
        panel.set_margin_end(2);
        panel.set_margin_top(2);
        panel.set_margin_bottom(2);

        // Animation controls
        let controls_box = GtkBox::new(Orientation::Horizontal, 2);
        
        // Create theme for animation buttons
        let anim_theme = Rc::new(RefCell::new(EditorTheme::default()));
        
        let play_btn = EditorButton::toolbar_icon("media-playback-start", "Play Animation", anim_theme.clone());
        let pause_btn = EditorButton::toolbar_icon("media-playback-pause", "Pause Animation", anim_theme.clone());
        let stop_btn = EditorButton::toolbar_icon("media-playback-stop", "Stop Animation", anim_theme.clone());
        let record_btn = EditorButton::new("🔴", ButtonVariant::Danger, ButtonSize::Small, anim_theme.clone());
        record_btn.widget().set_tooltip_text(Some("Record Animation"));
        
        controls_box.append(play_btn.widget());
        controls_box.append(pause_btn.widget());
        controls_box.append(stop_btn.widget());
        controls_box.append(record_btn.widget());
        
        // Timeline
        let timeline_label = Label::new(Some("Timeline:"));
        let timeline_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        timeline_scale.set_hexpand(true);
        timeline_scale.set_value(0.0);
        
        controls_box.append(&timeline_label);
        controls_box.append(&timeline_scale);
        
        panel.append(&controls_box);

        // Animation list
        let anim_frame = Frame::new(Some("Animations"));
        let anim_list = TreeView::new();
        let anim_store = TreeStore::new(&[Type::STRING, Type::STRING]);
        anim_list.set_model(Some(&anim_store));
        
        let name_column = TreeViewColumn::new();
        name_column.set_title("Animation");
        let name_renderer = CellRendererText::new();
        name_column.pack_start(&name_renderer, true);
        name_column.add_attribute(&name_renderer, "text", 0);
        anim_list.append_column(&name_column);
        
        // Add sample animations
        let idle_iter = anim_store.append(None);
        anim_store.set_value(&idle_iter, 0, &"Idle".to_value());
        let walk_iter = anim_store.append(None);
        anim_store.set_value(&walk_iter, 0, &"Walk".to_value());
        let run_iter = anim_store.append(None);
        anim_store.set_value(&run_iter, 0, &"Run".to_value());
        
        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&anim_list));
        scrolled.set_vexpand(true);
        anim_frame.set_child(Some(&scrolled));
        panel.append(&anim_frame);

        panel
    }

    fn create_audio_mixer_panel(&self) -> GtkBox {
        let panel = GtkBox::new(Orientation::Vertical, 2);
        panel.set_margin_start(2);
        panel.set_margin_end(2);
        panel.set_margin_top(2);
        panel.set_margin_bottom(2);

        // Master volume
        let master_frame = Frame::new(Some("Master"));
        let master_box = GtkBox::new(Orientation::Vertical, 2);
        master_box.set_margin_start(2);
        master_box.set_margin_end(2);
        master_box.set_margin_top(2);
        master_box.set_margin_bottom(2);

        let master_volume = Scale::with_range(Orientation::Vertical, 0.0, 100.0, 1.0);
        master_volume.set_value(80.0);
        master_volume.set_vexpand(true);
        master_volume.set_inverted(true); // Top = max volume
        
        let master_label = Label::new(Some("80%"));
        master_box.append(&master_volume);
        master_box.append(&master_label);
        master_frame.set_child(Some(&master_box));

        // Audio groups
        let groups_box = GtkBox::new(Orientation::Horizontal, 2);
        
        for (name, value) in [("Music", 70), ("SFX", 85), ("Voice", 90)] {
            let group_frame = Frame::new(Some(name));
            let group_box = GtkBox::new(Orientation::Vertical, 2);
            group_box.set_margin_start(2);
            group_box.set_margin_end(2);
            group_box.set_margin_top(2);
            group_box.set_margin_bottom(2);

            let group_volume = Scale::with_range(Orientation::Vertical, 0.0, 100.0, 1.0);
            group_volume.set_value(value as f64);
            group_volume.set_vexpand(true);
            group_volume.set_inverted(true);
            
            let group_label = Label::new(Some(&format!("{}%", value)));
            let mute_btn = CheckButton::with_label("Mute");
            
            group_box.append(&group_volume);
            group_box.append(&group_label);
            group_box.append(&mute_btn);
            group_frame.set_child(Some(&group_box));
            groups_box.append(&group_frame);
        }

        let main_layout = GtkBox::new(Orientation::Horizontal, 2);
        main_layout.append(&master_frame);
        main_layout.append(&groups_box);
        
        panel.append(&main_layout);

        panel
    }

    fn build_ui(&self, application: &AdwApplication) {
        // Create main window
        let window = AdwApplicationWindow::builder()
            .application(application)
            .title("Unity Editor - Mobile Game Engine (GTK4)")
            .default_width(1600)
            .default_height(1000)
            .build();

        // Load and apply custom design constraints after window is created
        println!("🎨 Loading design constraints...");
        let _constraints = engine_ui_system::apply_constraints_to_window(&window);
        engine_ui_system::preview_current_constraints();
        
        // Set up hot reload for design constraints
        self.setup_hot_reload(&window);
        
        window.set_resizable(true);
        
        // Move window controls to the left (macOS-style)
        window.set_decorated(true);
        if let Some(settings) = gtk4::Settings::default() {
            settings.set_property("gtk-decoration-layout", "close,minimize,maximize:");
        }

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 0);

        // Create header bar with left-side controls
        let header_bar = AdwHeaderBar::new();
        header_bar.set_title_widget(Some(&Label::new(Some("Unity Editor"))));
        header_bar.set_decoration_layout(Some("close,minimize,maximize:"));

        // Set up global menu bar (macOS style)
        let menu_model = Self::create_menu_model();
        application.set_menubar(Some(&menu_model));

        // Add header bar to main box instead of setting as titlebar
        main_box.append(&header_bar);

        // Add Unity-style toolbar at the top of the main window
        let unity_toolbar = self.create_unity_toolbar();
        main_box.append(&unity_toolbar);

        // Create main layout with paned windows
        let main_paned = Paned::new(Orientation::Horizontal);
        main_paned.set_hexpand(true);
        main_paned.set_vexpand(true);
        main_paned.set_position(250);
        main_paned.set_resize_start_child(true);
        main_paned.set_resize_end_child(true);
        main_paned.set_shrink_start_child(false);
        main_paned.set_shrink_end_child(false);

        // Left side - Hierarchy
        let hierarchy_panel = self.create_hierarchy_panel(&window);
        hierarchy_panel.set_size_request(200, -1); // Minimum width
        hierarchy_panel.set_hexpand(false);
        main_paned.set_start_child(Some(&hierarchy_panel));

        // Center and right layout
        let center_paned = Paned::new(Orientation::Horizontal);
        center_paned.set_hexpand(true);
        center_paned.set_vexpand(true);
        center_paned.set_position(800);
        center_paned.set_resize_start_child(true);
        center_paned.set_resize_end_child(true);
        center_paned.set_shrink_start_child(false);
        center_paned.set_shrink_end_child(false);
        
        // Center - Scene/Game view and bottom panels
        let center_vertical_paned = Paned::new(Orientation::Vertical);
        center_vertical_paned.set_hexpand(true);
        center_vertical_paned.set_vexpand(true);
        center_vertical_paned.set_position(600);
        center_vertical_paned.set_resize_start_child(true);
        center_vertical_paned.set_resize_end_child(true);
        center_vertical_paned.set_shrink_start_child(false);
        center_vertical_paned.set_shrink_end_child(false);
        
        // Scene/Game view with custom panel-style header with tab functionality
        let scene_container = GtkBox::new(Orientation::Vertical, 0);
        scene_container.set_hexpand(true);
        scene_container.set_vexpand(true);
        
        // Create panel with tabbed header
        let scene_panel = GtkBox::new(Orientation::Vertical, 0);
        scene_panel.add_css_class("standard-panel");
        
        let (header, tab_container) = self.create_tabbed_header("Scene");
        scene_panel.append(&header);
        
        // Add separator
        let separator = self.create_standard_separator();
        scene_panel.append(&separator);
        
        let scene_content = self.create_standard_content_area();
        scene_panel.append(&scene_content);
        
        // Add custom tab buttons
        let tab_theme = Rc::new(RefCell::new(EditorTheme::default()));
        let scene_tab_btn = EditorButton::new("Scene", ButtonVariant::Ghost, ButtonSize::Small, tab_theme.clone());
        let game_tab_btn = EditorButton::new("Game", ButtonVariant::Ghost, ButtonSize::Small, tab_theme.clone());
        
        // Style as tabs
        scene_tab_btn.widget().add_css_class("tab-button");
        scene_tab_btn.widget().add_css_class("tab-active");
        game_tab_btn.widget().add_css_class("tab-button");
        
        // Add tabs to tab container
        tab_container.append(scene_tab_btn.widget());
        tab_container.append(game_tab_btn.widget());
        
        // Create both views
        let scene_view = self.create_scene_view();
        let game_view = self.create_game_view();
        
        // Initially show Scene view
        scene_content.append(&scene_view);
        
        // Connect tab switching functionality
        let scene_content_clone = scene_content.clone();
        let scene_view_clone = scene_view.clone();
        let game_view_clone = game_view.clone();
        let scene_tab_clone = scene_tab_btn.widget().clone();
        let game_tab_clone = game_tab_btn.widget().clone();
        
        scene_tab_btn.widget().connect_clicked(move |_| {
            // Remove game view and add scene view
            scene_content_clone.remove(&game_view_clone);
            scene_content_clone.append(&scene_view_clone);
            
            // Update tab styling
            scene_tab_clone.add_css_class("tab-active");
            game_tab_clone.remove_css_class("tab-active");
        });
        
        let scene_content_clone2 = scene_content.clone();
        let scene_view_clone2 = scene_view.clone();
        let game_view_clone2 = game_view.clone();
        let scene_tab_clone2 = scene_tab_btn.widget().clone();
        let game_tab_clone2 = game_tab_btn.widget().clone();
        
        game_tab_btn.widget().connect_clicked(move |_| {
            // Remove scene view and add game view
            scene_content_clone2.remove(&scene_view_clone2);
            scene_content_clone2.append(&game_view_clone2);
            
            // Update tab styling
            game_tab_clone2.add_css_class("tab-active");
            scene_tab_clone2.remove_css_class("tab-active");
        });
        
        // Add the panel to container  
        scene_container.append(&scene_panel);
        
        center_vertical_paned.set_start_child(Some(&scene_container));

        // Bottom panels with tabs
        let bottom_notebook = Notebook::new();
        bottom_notebook.set_size_request(-1, 150); // Minimum height
        bottom_notebook.set_hexpand(true);
        bottom_notebook.set_vexpand(false);

        // Console tab
        let console_panel = self.create_console_panel(&window);
        let console_label = Label::new(Some("Console"));
        bottom_notebook.append_page(&console_panel, Some(&console_label));

        // Project tab
        let project_panel = self.create_project_panel(&window);
        let project_label = Label::new(Some("Project"));
        bottom_notebook.append_page(&project_panel, Some(&project_label));

        // Animation tab
        let animation_panel = self.create_animation_panel();
        let animation_label = Label::new(Some("Animation"));
        bottom_notebook.append_page(&animation_panel, Some(&animation_label));

        // Audio Mixer tab
        let audio_panel = self.create_audio_mixer_panel();
        let audio_label = Label::new(Some("Audio Mixer"));
        bottom_notebook.append_page(&audio_panel, Some(&audio_label));

        center_vertical_paned.set_end_child(Some(&bottom_notebook));
        center_paned.set_start_child(Some(&center_vertical_paned));

        // Right side - Inspector
        let inspector_panel = self.create_inspector_panel(&window);
        inspector_panel.set_size_request(250, -1); // Minimum width
        inspector_panel.set_hexpand(false);
        center_paned.set_end_child(Some(&inspector_panel));

        main_paned.set_end_child(Some(&center_paned));

        main_box.append(&main_paned);
        window.set_content(Some(&main_box));

        // Setup drop zones for panel docking
        Self::setup_drop_zones(&window, &self.detachable_panels);

        // Apply custom CSS for better styling
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(
            "
            .error { color: #ff6b6b; }
            .success { color: #51cf66; }
            .accent { color: #339af0; }
            .monospace { font-family: var(--font-mono, monospace); }
            .toolbar { 
                background: var(--toolbar-background, transparent); 
                border: none;
                padding: var(--space-sm, 4px) var(--space-md, 8px);
                margin: 0;
                min-height: var(--toolbar-height, var(--panel-header-height, 32px));
                max-height: var(--toolbar-height, var(--panel-header-height, 32px));
                box-sizing: border-box;
                border-radius: 0;
            }
            
            .toolbar button {
                margin: 0;
                padding: 2px var(--space-sm, 4px);
                border: var(--border-width, 1px) solid var(--border-primary, #404040);
                background: var(--button-secondary-bg, var(--button-secondary, #404040));
            }
            
            /* Compact UI styling - exclude system buttons */
            button:not(.titlebutton):not(.close):not(.minimize):not(.maximize) {
                min-height: var(--button-height, 20px);
                min-width: var(--button-height, 20px);
                padding: var(--space-sm, 2px) var(--space-md, 6px);
                margin: var(--space-sm, 1px);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                border-radius: var(--border-radius, 4px);
                background: var(--button-primary, #007AFF);
                color: var(--text-primary, #FFFFFF);
                border: var(--border-width, 1px) solid var(--border-primary, #404040);
            }
            
            button.circular:not(.titlebutton) {
                min-height: var(--button-height, 18px);
                min-width: var(--button-height, 18px);
                padding: 0;
                border-radius: 50%;
            }
            
            entry {
                min-height: var(--input-height, 18px);
                padding: var(--space-sm, 2px) var(--space-sm, 4px);
                margin: var(--space-sm, 1px);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                border-radius: var(--border-radius, 4px);
                background: var(--input-background, var(--input-bg, #3A3A3A));
                color: var(--text-primary, #FFFFFF);
                border: var(--border-width, 1px) solid var(--input-border, var(--border-primary, #404040));
            }
            
            spinbutton {
                min-height: var(--input-height, 18px);
                padding: var(--space-sm, 1px) var(--space-sm, 3px);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                border-radius: var(--border-radius, 4px);
                background: var(--input-background, var(--input-bg, #3A3A3A));
                color: var(--text-primary, #FFFFFF);
                border: var(--border-width, 1px) solid var(--input-border, var(--border-primary, #404040));
            }
            
            label {
                padding: var(--space-sm, 1px);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                color: var(--text-primary, #FFFFFF);
            }
            
            /* Unified ScrolledWindow styling for all panels */
            scrolledwindow {
                border: none;
                background: transparent;
                border-radius: 0;
                padding: 0;
                margin: 0;
            }
            
            scrolledwindow > viewport {
                border: none;
                background: transparent;
                border-radius: 0;
            }
            
            /* Unified TreeView styling */
            treeview {
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                background: var(--panel-background, var(--panel-bg, #2D2D2D));
                color: var(--text-primary, #FFFFFF);
                border-radius: 0;
                border: none;
                padding: 0;
            }
            
            /* Unified TextView styling */
            textview {
                padding: var(--panel-padding, var(--space-sm, 3px));
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-mono, monospace);
                background: var(--panel-background, var(--panel-bg, #2D2D2D));
                color: var(--text-primary, #FFFFFF);
                border-radius: 0;
                border: none;
            }
            
            /* Make notebook tabs look like standard panel headers */
            notebook > header {
                padding: 0;
                margin: 0;
                min-height: var(--panel-header-height, 32px);
                max-height: var(--panel-header-height, 32px);
                background: transparent;
                border: none;
            }
            
            notebook > header > tabs {
                background: transparent;
                border: none;
                margin: 0;
                padding: 0;
            }
            
            notebook > header > tabs > tab {
                padding: var(--space-sm, 4px) var(--space-md, 8px);
                min-height: var(--panel-header-height, 32px);
                max-height: var(--panel-header-height, 32px);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                border-radius: 0;
                background: transparent;
                color: var(--text-secondary, #ABABAB);
                border: none;
                margin: 0;
                box-sizing: border-box;
            }
            
            notebook > header > tabs > tab:checked {
                background: transparent;
                color: var(--text-primary, #FFFFFF);
                border: none;
                border-top-left-radius: 0;
                border-top-right-radius: 0;
                position: relative;
                z-index: 1;
                font-weight: var(--font-weight-normal, 400);
            }
            
            notebook {
                border: none;
                background: transparent;
                margin: 0;
                padding: 0;
            }
            
            notebook > stack {
                border: none;
                background: var(--panel-background, var(--panel-bg, #2D2D2D));
                margin: 0;
                padding: 0;
                border-radius: var(--border-radius, 4px);
                border-top-left-radius: 0;
            }
            
            frame {
                padding: var(--panel-padding, var(--space-sm, 3px));
                margin: var(--space-lg, var(--space-sm, 2px));
                border-radius: var(--border-radius, 4px);
                background: var(--panel-background, var(--panel-bg, #2D2D2D));
                border: var(--border-width, 1px) solid var(--border-primary, #404040);
            }
            
            frame > label {
                font-weight: var(--font-weight-normal, bold);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                color: var(--text-primary, #FFFFFF);
            }
            
            box {
                spacing: var(--space-sm, 2px);
                margin: var(--space-sm, 2px);
            }
            
            /* Unified Panel System */
            .standard-panel {
                margin: 0;
                padding: 0;
                spacing: 0;
                background: var(--panel-background, var(--panel-bg, #2D2D2D));
            }
            
            .panel-header {
                padding: var(--space-sm, 4px) var(--space-md, 8px);
                margin: 0;
                spacing: 0;
                background: transparent;
                min-height: var(--panel-header-height, 32px);
                max-height: var(--panel-header-height, 32px);
                box-sizing: border-box;
                border: none;
                border-radius: 0;
            }
            
            .panel-title {
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                color: var(--text-primary, #FFFFFF);
                font-weight: var(--font-weight-normal, 400);
                line-height: 1.2;
            }
            
            .panel-separator {
                margin: 0;
                padding: 0;
                background: var(--border-primary, #404040);
                min-height: 1px;
            }
            
            .panel-content {
                padding: var(--panel-padding, var(--space-sm, 4px));
                margin: 0;
                spacing: var(--space-sm, 4px);
            }
            
            .panel-toolbar {
                padding: var(--space-sm, 4px);
                margin: 0 0 var(--space-sm, 4px) 0;
                spacing: var(--space-sm, 4px);
                background: transparent;
            }
            
            /* Legacy panel class for backward compatibility */
            .panel {
                margin: var(--space-sm, 2px);
                spacing: var(--space-sm, 2px);
            }
            
            /* Sidebar-specific styling - only apply to panel backgrounds, not headers */
            .sidebar {
                background: var(--sidebar-background, var(--panel-bg, #252525));
                min-width: var(--sidebar-width, 250px);
            }
            
            /* Ensure all panel headers have consistent transparent background */
            .panel-header {
                background: transparent !important;
            }
            
            /* Force notebook tabs to have same background as panel headers */
            notebook > header {
                background: transparent !important;
            }
            
            notebook > header > tabs {
                background: transparent !important;
            }
            
            notebook > header > tabs > tab {
                background: transparent !important;
            }
            
            notebook > header > tabs > tab:checked {
                background: transparent !important;
            }
            
            /* Custom tab styling for Scene/Game tabs */
            .tab-container {
                margin: 0;
                padding: 0;
                spacing: 0;
            }
            
            .tab-button {
                min-height: var(--panel-header-height, 32px);
                max-height: var(--panel-header-height, 32px);
                padding: var(--space-sm, 4px) var(--space-md, 12px);
                margin: 0;
                border: none;
                border-radius: 0;
                background: transparent;
                color: var(--text-secondary, #ABABAB);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                font-weight: var(--font-weight-normal, 400);
                border-bottom: 2px solid transparent;
            }
            
            .tab-button:hover {
                background: var(--panel-background, var(--panel-bg, #2D2D2D));
                color: var(--text-primary, #FFFFFF);
            }
            
            .tab-button.tab-active {
                color: var(--text-primary, #FFFFFF);
                font-weight: var(--font-weight-medium, 500);
                background: transparent;
                border-bottom: 2px solid var(--button-primary, #007AFF);
            }
            
            /* Panel action buttons (detach, etc.) */
            .panel-action-btn {
                min-width: 20px;
                min-height: 20px;
                padding: 2px 4px;
                margin: 0 2px;
                border: none;
                border-radius: var(--border-radius, 4px);
                background: transparent;
                color: var(--text-secondary, #ABABAB);
                font-size: var(--font-size-sm, 11px);
                opacity: 0.7;
            }
            
            .panel-action-btn:hover {
                background: var(--button-secondary-hover, #4A4A4A);
                color: var(--text-primary, #FFFFFF);
                opacity: 1.0;
            }
            
            /* Panel placeholder when detached */
            .panel-placeholder {
                background: var(--panel-background, var(--panel-bg, #2D2D2D));
                border: 2px dashed var(--border-primary, #404040);
                min-width: 200px;
                min-height: 150px;
            }
            
            /* Draggable panel headers */
            .draggable-header {
                cursor: grab;
            }
            
            .draggable-header:active {
                cursor: grabbing;
            }
            
            /* Drop zone indicators */
            .drop-zone-active {
                background: var(--button-primary, #007AFF);
                opacity: 0.3;
                border: 2px solid var(--button-primary, #007AFF);
            }
            
            .drop-zone-highlight {
                background: var(--button-primary-hover, #0056CC);
                opacity: 0.5;
                transition: all 0.2s ease;
            }
            
            /* Drag icon styling */
            .drag-icon {
                background: var(--panel-background, #2D2D2D);
                color: var(--text-primary, #FFFFFF);
                border: 1px solid var(--border-primary, #404040);
                border-radius: var(--border-radius, 4px);
                padding: 4px 8px;
                font-size: var(--font-size-sm, 11px);
                opacity: 0.9;
            }
            
            /* Floating drag panel styling */
            .floating-drag-panel {
                background: var(--panel-background, #2D2D2D);
                color: var(--text-primary, #FFFFFF);
                border: 2px solid var(--button-primary, #007AFF);
                border-radius: var(--border-radius, 6px);
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
                opacity: 0.95;
            }
            
            /* In-window floating panel */
            .floating-panel {
                background: var(--panel-background, #2D2D2D);
                color: var(--text-primary, #FFFFFF);
                border: 2px solid var(--button-primary, #007AFF);
                border-radius: var(--border-radius, 6px);
                box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
                opacity: 0.9;
                transition: all 0.1s ease;
            }
            
            .floating-panel label {
                color: var(--text-primary, #FFFFFF);
                font-size: var(--font-size-base, 12px);
                padding: 8px;
            }
            
            .drag-title {
                font-weight: var(--font-weight-medium, 500);
                font-size: var(--font-size-sm, 11px);
                color: var(--text-primary, #FFFFFF);
            }
            
            combobox {
                min-height: var(--input-height, 18px);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                border-radius: var(--border-radius, 4px);
                background: var(--input-bg, #3A3A3A);
                color: var(--text-primary, #FFFFFF);
                border: var(--border-width, 1px) solid var(--border-primary, #404040);
            }
            
            scale {
                min-height: var(--input-height, 12px);
                border-radius: var(--border-radius, 4px);
            }
            
            /* Exclude system headerbar and window controls from custom styling */
            headerbar {
                /* Let system handle headerbar styling */
            }
            
            /* Preserve system window control buttons */
            headerbar button.titlebutton,
            button.titlebutton,
            .titlebutton,
            window > headerbar button {
                all: revert !important;
                font-family: inherit !important;
                font-size: inherit !important;
                padding: inherit !important;
                margin: inherit !important;
                border: inherit !important;
                background: inherit !important;
                color: inherit !important;
                border-radius: inherit !important;
            }
            
            menubutton {
                min-height: var(--button-height, 20px);
                min-width: var(--button-height, 20px);
                padding: var(--space-sm, 2px);
                font-size: var(--font-size-base, 12px);
                font-family: var(--font-primary, sans-serif);
                border-radius: var(--border-radius, 4px);
                background: var(--button-primary, #007AFF);
                color: var(--text-primary, #FFFFFF);
                border: var(--border-width, 1px) solid var(--border-primary, #404040);
            }
            
            /* Unity-style toolbar at top of main window */
            .unity-toolbar {
                background: var(--toolbar-background, #2D2D2D);
                border-bottom: 1px solid var(--border-primary, #404040);
                min-height: var(--toolbar-height, 42px);
                max-height: var(--toolbar-height, 42px);
                padding: var(--space-sm, 4px) var(--space-md, 8px);
                margin: 0;
                spacing: var(--space-md, 8px);
            }
            
            .toolbar-btn {
                min-width: 32px;
                min-height: 32px;
                margin: 2px;
                padding: 6px;
                border-radius: var(--border-radius-sm, 4px);
                font-size: var(--font-size-md, 14px);
            }
            
            .scene-header {
                background: var(--panel-background, #2D2D2D);
                border-bottom: 1px solid var(--border-primary, #404040);
                padding: var(--space-sm, 4px);
            }
            
            .scene-header button {
                min-width: 24px;
                min-height: 24px;
            }
            "
        );

        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&window),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        window.present();
    }

    fn create_unity_toolbar(&self) -> GtkBox {
        // Create the Unity-style toolbar container
        let toolbar_box = GtkBox::new(Orientation::Horizontal, 8);
        toolbar_box.set_margin_start(8);
        toolbar_box.set_margin_end(8);
        toolbar_box.set_margin_top(6);
        toolbar_box.set_margin_bottom(6);
        toolbar_box.add_css_class("unity-toolbar");

        // Create theme for toolbar buttons
        let toolbar_theme = Rc::new(RefCell::new(EditorTheme::default()));

        // Tool selection buttons (left side)
        let tools_box = GtkBox::new(Orientation::Horizontal, 4);
        
        let select_btn = EditorButton::toolbar_icon("edit-select", "Select Tool", toolbar_theme.clone());
        let move_btn = EditorButton::toolbar_icon("transform-move", "Move Tool", toolbar_theme.clone());
        let rotate_btn = EditorButton::toolbar_icon("transform-rotate", "Rotate Tool", toolbar_theme.clone());
        let scale_btn = EditorButton::toolbar_icon("transform-scale", "Scale Tool", toolbar_theme.clone());

        tools_box.append(select_btn.widget());
        tools_box.append(move_btn.widget());
        tools_box.append(rotate_btn.widget());
        tools_box.append(scale_btn.widget());

        // Central spacer
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        // Play controls (center)
        let play_controls_box = GtkBox::new(Orientation::Horizontal, 4);
        
        let play_btn = EditorButton::new("▶", ButtonVariant::Primary, ButtonSize::Small, toolbar_theme.clone());
        play_btn.widget().set_tooltip_text(Some("Play"));
        let pause_btn = EditorButton::toolbar_icon("media-playback-pause", "Pause", toolbar_theme.clone());
        let stop_btn = EditorButton::new("⏹", ButtonVariant::Danger, ButtonSize::Small, toolbar_theme.clone());
        stop_btn.widget().set_tooltip_text(Some("Stop"));

        play_controls_box.append(play_btn.widget());
        play_controls_box.append(pause_btn.widget());
        play_controls_box.append(stop_btn.widget());

        // Right spacer
        let spacer2 = GtkBox::new(Orientation::Horizontal, 0);
        spacer2.set_hexpand(true);

        // Account/cloud controls (right side)
        let account_box = GtkBox::new(Orientation::Horizontal, 4);
        
        let cloud_btn = EditorButton::toolbar_icon("weather-few-clouds", "Cloud Build", toolbar_theme.clone());
        let account_btn = EditorButton::toolbar_icon("system-users", "Account", toolbar_theme.clone());

        account_box.append(cloud_btn.widget());
        account_box.append(account_btn.widget());

        // Assemble toolbar
        toolbar_box.append(&tools_box);
        toolbar_box.append(&spacer);
        toolbar_box.append(&play_controls_box);
        toolbar_box.append(&spacer2);
        toolbar_box.append(&account_box);

        println!("🔧 Created Unity-style toolbar at top of main window");
        
        toolbar_box
    }

    fn run(&self) {
        self.setup_actions();

        let editor_state = self.editor_state.clone();
        self.app.connect_activate(move |app| {
            let unity_app = UnityEditorApp {
                app: app.clone(),
                editor_state: editor_state.clone(),
                toolbar_window: None,
                detachable_panels: Rc::new(RefCell::new(std::collections::HashMap::new())),
            };
            unity_app.build_ui(app);
        });

        let args: Vec<String> = std::env::args().collect();
        self.app.run_with_args(&args);
    }
}

fn main() -> glib::ExitCode {
    println!("🎮 Mobile Game Engine - GTK4 Unity Editor");
    println!("🔧 Initializing GTK4 application...");

    let app = UnityEditorApp::new();
    
    println!("✅ GTK4 Unity editor initialized successfully");
    println!("🖥️  Starting GTK application...");

    app.run();
    
    glib::ExitCode::SUCCESS
}