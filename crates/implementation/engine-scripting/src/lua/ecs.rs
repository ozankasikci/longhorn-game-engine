//! ECS integration for Lua scripting

use mlua::{Lua, Table, Value, UserData, UserDataMethods};
use engine_ecs_core::{World, Entity};
use engine_component_traits::Component;
use crate::ScriptResult;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Wrapper for safe World access from Lua
pub struct LuaWorld {
    world: Arc<Mutex<World>>,
}

impl LuaWorld {
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self { world }
    }
}

impl UserData for LuaWorld {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Create entity method
        methods.add_method_mut("create_entity", |_lua, this, components: Table| {
            let mut world = this.world.lock().unwrap();
            let entity = world.spawn();
            
            // Add components from table
            for pair in components.pairs::<String, Table>() {
                let (component_name, component_table) = pair?;
                
                println!("Processing component: {}", component_name);
                
                // For now, only handle known components
                if component_name == "Transform" {
                    if let Ok(transform_component) = table_to_component(&component_table, "Transform") {
                        // Downcast to concrete Transform type and add to entity
                        if let Some(transform) = transform_component.as_any().downcast_ref::<crate::components::Transform>() {
                            let transform_clone = transform.clone();
                            println!("Adding Transform component: {:?}", transform_clone);
                            if let Err(e) = world.add_component(entity, transform_clone) {
                                println!("Failed to add Transform component: {:?}", e);
                            } else {
                                println!("Successfully added Transform component to entity {:?}", entity.id());
                            }
                        } else {
                            println!("Failed to downcast Transform component");
                        }
                    }
                } else if component_name == "Health" {
                    // For now, create a default health component since we don't have marshaling for it yet
                    let health = crate::components::Health { current: 100, max: 100 };
                    println!("Adding Health component: {:?}", health);
                    if let Err(e) = world.add_component(entity, health) {
                        println!("Failed to add Health component: {:?}", e);
                    } else {
                        println!("Successfully added Health component to entity {:?}", entity.id());
                    }
                } else if component_name == "LuaScript" {
                    // Parse LuaScript component from Lua table
                    let script_path: String = component_table.get("script_path").unwrap_or_else(|_| "".to_string());
                    let enabled: bool = component_table.get("enabled").unwrap_or(true);
                    let execution_order: i32 = component_table.get("execution_order").unwrap_or(0);
                    
                    let lua_script = crate::components::LuaScript {
                        script_path,
                        enabled,
                        instance_id: None,
                        execution_order,
                    };
                    println!("Adding LuaScript component: {:?}", lua_script);
                    if let Err(e) = world.add_component(entity, lua_script) {
                        println!("Failed to add LuaScript component: {:?}", e);
                    } else {
                        println!("Successfully added LuaScript component to entity {:?}", entity.id());
                    }
                }
                // Add other component types as needed
            }
            
            // Return a LuaEntity wrapper instead of just the ID
            let lua_entity = LuaEntity {
                entity,
                world: this.world.clone(),
            };
            Ok(lua_entity)
        });

        // Query entities method - return iterator for Lua for loops
        methods.add_method("query", |lua, this, component_name: String| {
            let world = this.world.lock().unwrap();
            
            // Currently only support Transform queries
            if component_name == "Transform" {
                // Collect all results upfront
                let mut results = Vec::new();
                for (entity, transform) in world.query_legacy::<crate::components::Transform>() {
                    let entity_id = entity.id() as u64;
                    let transform_clone = transform.clone();
                    results.push((entity_id, transform_clone));
                }
                drop(world);
                
                // Create iterator state using a table
                let state_table = lua.create_table()?;
                state_table.set("index", 1)?;
                state_table.set("results", lua.create_table()?)?;
                
                // Store results in the state table
                let results_table: mlua::Table = state_table.get("results")?;
                for (i, (entity_id, transform)) in results.iter().enumerate() {
                    let entry = lua.create_table()?;
                    entry.set("entity_id", *entity_id)?;
                    let transform_table = component_to_table(lua, transform)
                        .map_err(|e| mlua::Error::RuntimeError(format!("Component conversion error: {:?}", e)))?;
                    entry.set("transform", transform_table)?;
                    results_table.set(i + 1, entry)?; // Lua arrays are 1-indexed
                }
                
                let world_arc = this.world.clone();
                
                // Return iterator function that Lua can use in for loops
                let iterator = lua.create_function(move |lua, _: ()| {
                    let index: usize = state_table.get("index")?;
                    let results_table: mlua::Table = state_table.get("results")?;
                    
                    if let Ok(entry) = results_table.get::<mlua::Table>(index) {
                        // Update index for next iteration
                        state_table.set("index", index + 1)?;
                        
                        let entity_id: u64 = entry.get("entity_id")?;
                        let transform_table: mlua::Table = entry.get("transform")?;
                        
                        // Create LuaEntity wrapper
                        let lua_entity = LuaEntity {
                            entity: engine_ecs_core::Entity::new(entity_id as u32, 1),
                            world: world_arc.clone(),
                        };
                        
                        // Return as MultiValue (entity, transform)
                        let mut values = mlua::MultiValue::new();
                        values.push_back(Value::UserData(lua.create_userdata(lua_entity)?));
                        values.push_back(Value::Table(transform_table));
                        Ok(values)
                    } else {
                        // End of iteration - return nil values
                        let mut values = mlua::MultiValue::new();
                        values.push_back(Value::Nil);
                        values.push_back(Value::Nil);
                        Ok(values)
                    }
                })?;
                
                Ok(iterator)
            } else {
                // Empty iterator for unsupported components
                let iterator = lua.create_function(|_, _: ()| {
                    let mut values = mlua::MultiValue::new();
                    values.push_back(Value::Nil);
                    values.push_back(Value::Nil);
                    Ok(values)
                })?;
                Ok(iterator)
            }
        });

        // Get entity method
        methods.add_method("get_entity", |_lua, _this, _entity_id: u64| {
            // let _world = this.world.lock().unwrap();
            
            // TODO: Return entity wrapper
            Ok(Value::Nil)
        });
    }
}

/// Wrapper for Entity access from Lua
#[derive(Clone)]
pub struct LuaEntity {
    pub entity: Entity,
    pub world: Arc<Mutex<World>>,
}

impl UserData for LuaEntity {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Get component method
        methods.add_method("get_component", |lua, this, component_type: String| {
            match component_type.as_str() {
                "Transform" => {
                    // First try to get from shared state (for script tests)
                    if let Some(transform) = crate::shared_state::get_entity_transform(this.entity) {
                        match component_to_table(lua, &transform) {
                            Ok(table) => return Ok(Value::Table(table)),
                            Err(e) => {
                                log::error!("Failed to convert Transform to table: {:?}", e);
                                return Ok(Value::Nil);
                            }
                        }
                    }
                    
                    // Fallback to world lookup
                    let world = this.world.lock().unwrap();
                    if let Some(transform) = world.get_component::<crate::components::Transform>(this.entity) {
                        // Convert Transform component to Lua table
                        match component_to_table(lua, transform) {
                            Ok(table) => Ok(Value::Table(table)),
                            Err(e) => {
                                log::error!("Failed to convert Transform to table: {:?}", e);
                                Ok(Value::Nil)
                            }
                        }
                    } else {
                        Ok(Value::Nil)
                    }
                }
                "Health" => {
                    let world = this.world.lock().unwrap();
                    if let Some(health) = world.get_component::<crate::components::Health>(this.entity) {
                        // Convert Health component to Lua table
                        let table = lua.create_table().map_err(|e| {
                            mlua::Error::RuntimeError(format!("Failed to create health table: {}", e))
                        })?;
                        table.set("current", health.current)?;
                        table.set("max", health.max)?;
                        Ok(Value::Table(table))
                    } else {
                        Ok(Value::Nil)
                    }
                }
                "LuaScript" => {
                    let world = this.world.lock().unwrap();
                    if let Some(lua_script) = world.get_component::<crate::components::LuaScript>(this.entity) {
                        // Convert LuaScript component to Lua table
                        let table = lua.create_table().map_err(|e| {
                            mlua::Error::RuntimeError(format!("Failed to create lua_script table: {}", e))
                        })?;
                        table.set("script_path", lua_script.script_path.clone())?;
                        table.set("enabled", lua_script.enabled)?;
                        table.set("execution_order", lua_script.execution_order)?;
                        if let Some(instance_id) = lua_script.instance_id {
                            table.set("instance_id", instance_id)?;
                        }
                        Ok(Value::Table(table))
                    } else {
                        Ok(Value::Nil)
                    }
                }
                "Velocity" => {
                    let world = this.world.lock().unwrap();
                    if let Some(velocity) = world.get_component::<crate::components::Velocity>(this.entity) {
                        // Convert Velocity component to Lua table
                        let table = lua.create_table().map_err(|e| {
                            mlua::Error::RuntimeError(format!("Failed to create velocity table: {}", e))
                        })?;
                        
                        // Create linear velocity table
                        let linear_table = lua.create_table()?;
                        linear_table.set("x", velocity.linear[0])?;
                        linear_table.set("y", velocity.linear[1])?;
                        linear_table.set("z", velocity.linear[2])?;
                        table.set("linear", linear_table)?;
                        
                        // Create angular velocity table
                        let angular_table = lua.create_table()?;
                        angular_table.set("x", velocity.angular[0])?;
                        angular_table.set("y", velocity.angular[1])?;
                        angular_table.set("z", velocity.angular[2])?;
                        table.set("angular", angular_table)?;
                        
                        Ok(Value::Table(table))
                    } else {
                        Ok(Value::Nil)
                    }
                }
                _ => {
                    Ok(Value::Nil)
                }
            }
        });

        // Add component method
        methods.add_method_mut("add_component", |lua, this, (component_type, data): (String, Table)| {
            let mut world = this.world.lock().unwrap();
            
            // Handle different component types
            match component_type.as_str() {
                "Transform" => {
                    if let Ok(transform_component) = table_to_component(&data, "Transform") {
                        if let Some(transform) = transform_component.as_any().downcast_ref::<crate::components::Transform>() {
                            let transform_clone = transform.clone();
                            world.add_component(this.entity, transform_clone).map_err(|e| {
                                mlua::Error::RuntimeError(format!("Failed to add Transform: {:?}", e))
                            })?;
                        }
                    }
                }
                "Health" => {
                    // Parse Health component from Lua table
                    let current: i32 = data.get("current").unwrap_or(100);
                    let max: i32 = data.get("max").unwrap_or(100);
                    let health = crate::components::Health { current, max };
                    world.add_component(this.entity, health).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to add Health: {:?}", e))
                    })?;
                }
                "LuaScript" => {
                    // Parse LuaScript component from Lua table
                    let script_path: String = data.get("script_path").unwrap_or_else(|_| "".to_string());
                    let enabled: bool = data.get("enabled").unwrap_or(true);
                    let execution_order: i32 = data.get("execution_order").unwrap_or(0);
                    let instance_id: Option<u64> = data.get("instance_id").ok();
                    
                    let lua_script = crate::components::LuaScript {
                        script_path,
                        enabled,
                        instance_id,
                        execution_order,
                    };
                    world.add_component(this.entity, lua_script).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to add LuaScript: {:?}", e))
                    })?;
                }
                "Velocity" => {
                    // Parse Velocity component from Lua table
                    let linear_table: Table = data.get("linear").unwrap_or_else(|_| {
                        // Create default table if not provided
                        lua.create_table().unwrap()
                    });
                    let angular_table: Table = data.get("angular").unwrap_or_else(|_| {
                        // Create default table if not provided
                        lua.create_table().unwrap()
                    });
                    
                    let linear_x: f32 = linear_table.get("x").unwrap_or(0.0);
                    let linear_y: f32 = linear_table.get("y").unwrap_or(0.0);
                    let linear_z: f32 = linear_table.get("z").unwrap_or(0.0);
                    
                    let angular_x: f32 = angular_table.get("x").unwrap_or(0.0);
                    let angular_y: f32 = angular_table.get("y").unwrap_or(0.0);
                    let angular_z: f32 = angular_table.get("z").unwrap_or(0.0);
                    
                    let velocity = crate::components::Velocity {
                        linear: [linear_x, linear_y, linear_z],
                        angular: [angular_x, angular_y, angular_z],
                    };
                    world.add_component(this.entity, velocity).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to add Velocity: {:?}", e))
                    })?;
                }
                _ => {
                    return Err(mlua::Error::RuntimeError(format!("Unsupported component type: {}", component_type)));
                }
            }
            
            Ok(())
        });

        // Set component method (update existing component)
        methods.add_method("set_component", |_lua, this, (component_type, data): (String, Table)| {
            match component_type.as_str() {
                "Transform" => {
                    // Parse Transform from Lua table and update shared state
                    if let Ok(transform_component) = table_to_component(&data, "Transform") {
                        if let Some(transform) = transform_component.as_any().downcast_ref::<crate::components::Transform>() {
                            // Update in shared state for script tests
                            crate::shared_state::update_entity_transform(this.entity, transform.clone());
                            
                            // Also try to update in the actual world if possible
                            if let Ok(mut world) = this.world.try_lock() {
                                if world.has_component::<crate::components::Transform>(this.entity) {
                                    if let Some(world_transform) = world.get_component_mut::<crate::components::Transform>(this.entity) {
                                        *world_transform = transform.clone();
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                }
                "Health" => {
                    // Parse Health component from Lua table and update world
                    let current: i32 = data.get("current").unwrap_or(100);
                    let max: i32 = data.get("max").unwrap_or(100);
                    let health = crate::components::Health { current, max };
                    
                    if let Ok(mut world) = this.world.try_lock() {
                        if world.has_component::<crate::components::Health>(this.entity) {
                            if let Some(world_health) = world.get_component_mut::<crate::components::Health>(this.entity) {
                                *world_health = health;
                            }
                        }
                    }
                    Ok(())
                }
                "Velocity" => {
                    // Parse Velocity component from Lua table and update world
                    let linear_table: Table = data.get("linear")?;
                    let angular_table: Table = data.get("angular")?;
                    
                    let linear_x: f32 = linear_table.get("x").unwrap_or(0.0);
                    let linear_y: f32 = linear_table.get("y").unwrap_or(0.0);
                    let linear_z: f32 = linear_table.get("z").unwrap_or(0.0);
                    
                    let angular_x: f32 = angular_table.get("x").unwrap_or(0.0);
                    let angular_y: f32 = angular_table.get("y").unwrap_or(0.0);
                    let angular_z: f32 = angular_table.get("z").unwrap_or(0.0);
                    
                    let velocity = crate::components::Velocity {
                        linear: [linear_x, linear_y, linear_z],
                        angular: [angular_x, angular_y, angular_z],
                    };
                    
                    if let Ok(mut world) = this.world.try_lock() {
                        if world.has_component::<crate::components::Velocity>(this.entity) {
                            if let Some(world_velocity) = world.get_component_mut::<crate::components::Velocity>(this.entity) {
                                *world_velocity = velocity;
                            }
                        }
                    }
                    Ok(())
                }
                _ => {
                    Err(mlua::Error::RuntimeError(format!("set_component not supported for type: {}", component_type)))
                }
            }
        });

        // Remove component method
        methods.add_method_mut("remove_component", |_lua, this, component_type: String| {
            let mut world = this.world.lock().unwrap();
            
            match component_type.as_str() {
                "Transform" => {
                    world.remove_component::<crate::components::Transform>(this.entity).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to remove Transform: {:?}", e))
                    })?;
                }
                "Health" => {
                    world.remove_component::<crate::components::Health>(this.entity).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to remove Health: {:?}", e))
                    })?;
                }
                "LuaScript" => {
                    world.remove_component::<crate::components::LuaScript>(this.entity).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to remove LuaScript: {:?}", e))
                    })?;
                }
                "Velocity" => {
                    world.remove_component::<crate::components::Velocity>(this.entity).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to remove Velocity: {:?}", e))
                    })?;
                }
                _ => {
                    return Err(mlua::Error::RuntimeError(format!("Unsupported component type: {}", component_type)));
                }
            }
            
            Ok(())
        });

        // Get ID method
        methods.add_method("id", |_, this, ()| {
            Ok(this.entity.id())
        });
    }
}

/// Component serializer trait
pub trait ComponentSerializer: Send + Sync {
    fn to_lua_table(&self, lua: &Lua, component: &dyn Component) -> Result<Table, mlua::Error>;
    fn from_lua_table(&self, table: &Table) -> Result<Box<dyn Component>, mlua::Error>;
}

/// Component registration for Lua
pub struct LuaComponentRegistry {
    /// Map from component name to type ID
    type_mappings: HashMap<String, std::any::TypeId>,
    /// Map from type ID to serializer
    serializers: HashMap<std::any::TypeId, Box<dyn ComponentSerializer>>,
}

impl LuaComponentRegistry {
    pub fn new() -> Self {
        Self {
            type_mappings: HashMap::new(),
            serializers: HashMap::new(),
        }
    }

    /// Register a component type for Lua access
    pub fn register_component<T: Component + 'static>(&mut self, name: &str) {
        let type_id = std::any::TypeId::of::<T>();
        self.type_mappings.insert(name.to_string(), type_id);
        // Note: Serializer would be added separately
    }
    
    /// Check if a component type is registered
    pub fn has_component(&self, name: &str) -> bool {
        self.type_mappings.contains_key(name)
    }
    
    /// Get type ID for a component name
    pub fn get_type_id(&self, name: &str) -> Option<std::any::TypeId> {
        self.type_mappings.get(name).copied()
    }
}

/// Set up ECS bindings in the engine table
pub fn setup_ecs_bindings(lua: &Lua, engine: &Table, world: Arc<Mutex<World>>) -> ScriptResult<()> {
    // Create world wrapper
    let lua_world = LuaWorld::new(world.clone());
    
    engine.set("world", lua_world)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set world: {}", e)))?;

    // Component constructors table
    let components = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create components table: {}", e)))?;

    // TODO: Add component constructors as they're registered
    
    engine.set("components", components)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set components table: {}", e)))?;

    Ok(())
}

/// Global component registry (for simplicity in tests)
pub static GLOBAL_COMPONENT_REGISTRY: Lazy<Mutex<LuaComponentRegistry>> = 
    Lazy::new(|| Mutex::new(LuaComponentRegistry::new()));

/// Helper to convert Lua table to component data
pub fn table_to_component(table: &Table, component_type: &str) -> ScriptResult<Box<dyn Component>> {
    // For now, manually handle Transform for testing
    // In a real implementation, this would use the registry's serializers
    
    match component_type {
        "Transform" => {
            // Parse position
            let pos_table: Table = table.get("position")
                .map_err(|_| crate::ScriptError::RuntimeError("Missing position field".to_string()))?;
            let pos_x: f32 = pos_table.get("x").unwrap_or(0.0);
            let pos_y: f32 = pos_table.get("y").unwrap_or(0.0);
            let pos_z: f32 = pos_table.get("z").unwrap_or(0.0);
            
            // Parse rotation
            let rot_table: Table = table.get("rotation")
                .map_err(|_| crate::ScriptError::RuntimeError("Missing rotation field".to_string()))?;
            let rot_x: f32 = rot_table.get("x").unwrap_or(0.0);
            let rot_y: f32 = rot_table.get("y").unwrap_or(0.0);
            let rot_z: f32 = rot_table.get("z").unwrap_or(0.0);
            
            // Parse scale
            let scale_table: Table = table.get("scale")
                .map_err(|_| crate::ScriptError::RuntimeError("Missing scale field".to_string()))?;
            let scale_x: f32 = scale_table.get("x").unwrap_or(1.0);
            let scale_y: f32 = scale_table.get("y").unwrap_or(1.0);
            let scale_z: f32 = scale_table.get("z").unwrap_or(1.0);
            
            Ok(Box::new(crate::components::Transform {
                position: [pos_x, pos_y, pos_z],
                rotation: [rot_x, rot_y, rot_z],
                scale: [scale_x, scale_y, scale_z],
            }))
        }
        _ => Err(crate::ScriptError::RuntimeError(format!("Unknown component type: {}", component_type)))
    }
}

/// Helper to convert component data to Lua table
pub fn component_to_table(lua: &Lua, component: &dyn Component) -> ScriptResult<Table> {
    let table = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create component table: {}", e)))?;
    
    // Try to downcast to known types
    if let Some(transform) = component.as_any().downcast_ref::<crate::components::Transform>() {
        // Create position table
        let pos_table = lua.create_table()
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create position table: {}", e)))?;
        pos_table.set("x", transform.position[0])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set x: {}", e)))?;
        pos_table.set("y", transform.position[1])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set y: {}", e)))?;
        pos_table.set("z", transform.position[2])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set z: {}", e)))?;
        table.set("position", pos_table)
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set position: {}", e)))?;
        
        // Create rotation table
        let rot_table = lua.create_table()
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create rotation table: {}", e)))?;
        rot_table.set("x", transform.rotation[0])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set rot x: {}", e)))?;
        rot_table.set("y", transform.rotation[1])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set rot y: {}", e)))?;
        rot_table.set("z", transform.rotation[2])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set rot z: {}", e)))?;
        table.set("rotation", rot_table)
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set rotation: {}", e)))?;
        
        // Create scale table
        let scale_table = lua.create_table()
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create scale table: {}", e)))?;
        scale_table.set("x", transform.scale[0])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set scale x: {}", e)))?;
        scale_table.set("y", transform.scale[1])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set scale y: {}", e)))?;
        scale_table.set("z", transform.scale[2])
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set scale z: {}", e)))?;
        table.set("scale", scale_table)
            .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set scale: {}", e)))?;
    }
    
    Ok(table)
}

