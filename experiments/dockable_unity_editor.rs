use mobile_game_engine::{GameObject as GameObj, EditorState, ConsoleMessage as ConsoleMsg, ConsoleMessageType};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
use std::collections::HashMap;

slint::include_modules!();

impl From<&GameObj> for GameObject {
    fn from(obj: &GameObj) -> Self {
        Self {
            id: obj.id as i32,
            name: obj.name.clone().into(),
            position_x: obj.transform.position[0],
            position_y: obj.transform.position[1],
            position_z: obj.transform.position[2],
            rotation_x: obj.transform.rotation[0],
            rotation_y: obj.transform.rotation[1],
            rotation_z: obj.transform.rotation[2],
            scale_x: obj.transform.scale[0],
            scale_y: obj.transform.scale[1],
            scale_z: obj.transform.scale[2],
            active: obj.active,
        }
    }
}

impl From<&ConsoleMsg> for ConsoleMessage {
    fn from(msg: &ConsoleMsg) -> Self {
        let msg_type = match msg.message_type {
            ConsoleMessageType::Info => "Info",
            ConsoleMessageType::Warning => "Warning", 
            ConsoleMessageType::Error => "Error",
        };
        
        Self {
            message: msg.message.clone().into(),
            message_type: msg_type.into(),
            timestamp: format!("{:.2}s", msg.timestamp.elapsed().as_secs_f32()).into(),
        }
    }
}

#[derive(Clone, Debug)]
struct PanelState {
    id: String,
    title: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    is_floating: bool,
    is_visible: bool,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            x: 100.0,
            y: 100.0,
            width: 300.0,
            height: 400.0,
            is_floating: false,
            is_visible: true,
        }
    }
}

struct DockableUnityEditor {
    ui: DockableUnityEditor,
    editor_state: EditorState,
    objects_model: Rc<VecModel<GameObject>>,
    messages_model: Rc<VecModel<ConsoleMessage>>,
    panel_states: HashMap<String, PanelState>,
}

impl DockableUnityEditor {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ui = DockableUnityEditor::new()?;
        let editor_state = EditorState::new();
        
        // Create models for dynamic data
        let objects_model = Rc::new(VecModel::default());
        let messages_model = Rc::new(VecModel::default());
        
        // Initialize panel states
        let mut panel_states = HashMap::new();
        
        panel_states.insert("hierarchy".to_string(), PanelState {
            id: "hierarchy".to_string(),
            title: "Hierarchy".to_string(),
            x: 100.0,
            y: 100.0,
            width: 250.0,
            height: 400.0,
            is_floating: false,
            is_visible: true,
        });
        
        panel_states.insert("inspector".to_string(), PanelState {
            id: "inspector".to_string(),
            title: "Inspector".to_string(),
            x: 200.0,
            y: 150.0,
            width: 300.0,
            height: 500.0,
            is_floating: false,
            is_visible: true,
        });
        
        panel_states.insert("console".to_string(), PanelState {
            id: "console".to_string(),
            title: "Console".to_string(),
            x: 300.0,
            y: 200.0,
            width: 600.0,
            height: 300.0,
            is_floating: false,
            is_visible: true,
        });
        
        panel_states.insert("project".to_string(), PanelState {
            id: "project".to_string(),
            title: "Project".to_string(),
            x: 400.0,
            y: 250.0,
            width: 400.0,
            height: 350.0,
            is_floating: false,
            is_visible: true,
        });
        
        // Set up the UI models
        ui.set_scene_objects(ModelRc::from(objects_model.clone()));
        ui.set_console_messages(ModelRc::from(messages_model.clone()));
        ui.set_selected_object_index(-1);
        
        // Set initial panel states
        ui.set_hierarchy_floating(false);
        ui.set_inspector_floating(false);
        ui.set_console_floating(false);
        ui.set_project_floating(false);
        
        let mut editor = Self {
            ui,
            editor_state,
            objects_model,
            messages_model,
            panel_states,
        };
        
        // Initialize with default objects and update UI
        editor.sync_objects_to_ui();
        editor.sync_messages_to_ui();
        editor.setup_callbacks();
        
        Ok(editor)
    }
    
    fn setup_callbacks(&self) {
        let ui_weak = self.ui.as_weak();
        let objects_model = self.objects_model.clone();
        let messages_model = self.messages_model.clone();
        
        // Object selection callback
        self.ui.on_object_selected({
            let ui_weak = ui_weak.clone();
            move |index| {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_selected_object_index(index);
                    println!("🎯 Selected object at index: {}", index);
                }
            }
        });
        
        // Object creation callback
        self.ui.on_object_created({
            let ui_weak = ui_weak.clone();
            let objects_model = objects_model.clone();
            move || {
                if let Some(_ui) = ui_weak.upgrade() {
                    let new_obj = GameObject {
                        id: objects_model.row_count() as i32 + 1,
                        name: format!("GameObject {}", objects_model.row_count() + 1).into(),
                        position_x: 0.0,
                        position_y: 0.0,
                        position_z: 0.0,
                        rotation_x: 0.0,
                        rotation_y: 0.0,
                        rotation_z: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                        scale_z: 1.0,
                        active: true,
                    };
                    
                    objects_model.push(new_obj);
                    println!("➕ Created new GameObject");
                }
            }
        });
        
        // Object deletion callback
        self.ui.on_object_deleted({
            let ui_weak = ui_weak.clone();
            let objects_model = objects_model.clone();
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let selected_index = ui.get_selected_object_index();
                    if selected_index >= 0 && (selected_index as usize) < objects_model.row_count() {
                        objects_model.remove(selected_index as usize);
                        ui.set_selected_object_index(-1);
                        println!("🗑️  Deleted object at index: {}", selected_index);
                    }
                }
            }
        });
        
        // Scene click callback
        self.ui.on_scene_clicked(move |x, y| {
            println!("🎮 Scene clicked at: ({:.1}, {:.1})", x, y);
        });
        
        // Tool selection callback
        self.ui.on_tool_selected(move |tool| {
            println!("🔧 Tool selected: {}", tool);
        });
        
        // Console clear callback
        self.ui.on_console_cleared({
            let messages_model = messages_model.clone();
            move || {
                messages_model.set_vec(vec![]);
                println!("🧹 Console cleared");
            }
        });
        
        // Asset selection callback
        self.ui.on_asset_selected(move |asset| {
            println!("📁 Asset selected: {}", asset);
        });
        
        // Property change callback
        self.ui.on_property_changed(move |property, value| {
            println!("⚙️  Property changed: {} = {}", property, value);
        });
        
        // Panel management callbacks
        self.ui.on_panel_closed({
            let ui_weak = ui_weak.clone();
            move |panel_id| {
                if let Some(_ui) = ui_weak.upgrade() {
                    println!("❌ Panel closed: {}", panel_id);
                    // TODO: Hide panel
                }
            }
        });
        
        self.ui.on_panel_docked({
            let ui_weak = ui_weak.clone();
            move |panel_id| {
                if let Some(ui) = ui_weak.upgrade() {
                    println!("⚓ Panel docked: {}", panel_id);
                    
                    match panel_id.as_str() {
                        "hierarchy" => ui.set_hierarchy_floating(false),
                        "inspector" => ui.set_inspector_floating(false),
                        "console" => ui.set_console_floating(false),
                        "project" => ui.set_project_floating(false),
                        _ => {}
                    }
                }
            }
        });
        
        self.ui.on_panel_undocked({
            let ui_weak = ui_weak.clone();
            move |panel_id| {
                if let Some(ui) = ui_weak.upgrade() {
                    println!("📌 Panel undocked: {}", panel_id);
                    
                    match panel_id.as_str() {
                        "hierarchy" => ui.set_hierarchy_floating(true),
                        "inspector" => ui.set_inspector_floating(true),
                        "console" => ui.set_console_floating(true),
                        "project" => ui.set_project_floating(true),
                        _ => {}
                    }
                }
            }
        });
        
        self.ui.on_panel_moved({
            let ui_weak = ui_weak.clone();
            move |panel_id, x, y| {
                if let Some(ui) = ui_weak.upgrade() {
                    println!("🚚 Panel moved: {} to ({:.1}, {:.1})", panel_id, x, y);
                    
                    match panel_id.as_str() {
                        "hierarchy" => {
                            ui.set_hierarchy_x(x);
                            ui.set_hierarchy_y(y);
                        },
                        "inspector" => {
                            ui.set_inspector_x(x);
                            ui.set_inspector_y(y);
                        },
                        "console" => {
                            ui.set_console_x(x);
                            ui.set_console_y(y);
                        },
                        "project" => {
                            ui.set_project_x(x);
                            ui.set_project_y(y);
                        },
                        _ => {}
                    }
                }
            }
        });
        
        self.ui.on_window_panel_requested({
            let ui_weak = ui_weak.clone();
            move |panel_id| {
                if let Some(ui) = ui_weak.upgrade() {
                    println!("🪟 Window panel requested: {}", panel_id);
                    
                    // Float the requested panel
                    match panel_id.as_str() {
                        "hierarchy" => ui.set_hierarchy_floating(true),
                        "inspector" => ui.set_inspector_floating(true),
                        "console" => ui.set_console_floating(true),
                        "project" => ui.set_project_floating(true),
                        _ => {}
                    }
                }
            }
        });
    }
    
    fn sync_objects_to_ui(&self) {
        let ui_objects: Vec<GameObject> = self.editor_state.scene_objects
            .values()
            .map(|obj| obj.into())
            .collect();
        
        self.objects_model.set_vec(ui_objects);
    }
    
    fn sync_messages_to_ui(&self) {
        let ui_messages: Vec<ConsoleMessage> = self.editor_state.console_messages
            .iter()
            .map(|msg| msg.into())
            .collect();
        
        self.messages_model.set_vec(ui_messages);
    }
    
    fn add_console_message(&self, message: &str, msg_type: ConsoleMessageType) {
        let ui_message = ConsoleMessage {
            message: message.into(),
            message_type: match msg_type {
                ConsoleMessageType::Info => "Info",
                ConsoleMessageType::Warning => "Warning",
                ConsoleMessageType::Error => "Error",
            }.into(),
            timestamp: "now".into(),
        };
        
        self.messages_model.push(ui_message);
    }
    
    fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Add some initial console messages
        self.add_console_message("🚀 Dockable Unity Editor started with Slint UI", ConsoleMessageType::Info);
        self.add_console_message("📋 Panels can be undocked by clicking the 📌 button", ConsoleMessageType::Info);
        self.add_console_message("🔗 Float panels can be docked by clicking the ⚓ button", ConsoleMessageType::Info);
        self.add_console_message("🎯 Drag floating panels by their title bar", ConsoleMessageType::Info);
        self.add_console_message("🪟 Use Window menu to create floating panels", ConsoleMessageType::Info);
        
        println!("🚀 Starting Dockable Slint Unity Editor");
        println!("📊 Loaded {} objects", self.objects_model.row_count());
        println!("💬 Console has {} messages", self.messages_model.row_count());
        println!("🔧 Features:");
        println!("   📌 Click undock button to float panels");
        println!("   ⚓ Click dock button to dock floating panels");
        println!("   🎯 Drag floating panels by title bar");
        println!("   🪟 Use Window menu to show hidden panels");
        
        // Run the UI
        self.ui.run()?;
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🎮 Mobile Game Engine - Dockable Unity Editor (Slint)");
    println!("🔧 Initializing dockable panel system...");
    
    let editor = DockableUnityEditor::new()?;
    
    println!("✅ Dockable editor initialized successfully");
    println!("🖥️  Starting UI event loop...");
    
    editor.run()
}