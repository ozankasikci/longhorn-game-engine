//! EditorClient - main interface for communicating with the editor.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use serde_json::json;

use crate::error::EditorError;
use crate::responses::*;

/// Default socket path for the editor.
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/longhorn-editor.sock";

/// Client for communicating with the Longhorn Editor via Unix socket.
pub struct EditorClient {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

impl EditorClient {
    /// Connect to the editor at the default socket path.
    pub fn connect_default() -> Result<Self, EditorError> {
        Self::connect(DEFAULT_SOCKET_PATH)
    }

    /// Connect to the editor at the specified socket path.
    pub fn connect(socket_path: impl AsRef<Path>) -> Result<Self, EditorError> {
        let stream = UnixStream::connect(socket_path)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream);

        Ok(Self { reader, writer })
    }

    /// Send a raw command and get the raw response.
    fn send_raw(&mut self, command: serde_json::Value) -> Result<RemoteResponse, EditorError> {
        let command_str = serde_json::to_string(&command)
            .map_err(EditorError::SerializeFailed)?;

        writeln!(self.writer, "{}", command_str)?;
        self.writer.flush()?;

        let mut line = String::new();
        self.reader.read_line(&mut line)?;

        let response: RemoteResponse = serde_json::from_str(&line)
            .map_err(EditorError::DeserializeFailed)?;

        if !response.ok {
            return Err(EditorError::EditorError(
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(response)
    }

    /// Parse response data into a specific type.
    fn parse_data<T: serde::de::DeserializeOwned>(response: RemoteResponse) -> Result<T, EditorError> {
        let data = response.data.ok_or(EditorError::MissingData)?;
        serde_json::from_value(data).map_err(EditorError::DeserializeFailed)
    }

    // ========== Connection & Utility ==========

    /// Ping the editor to verify the connection is alive.
    pub fn ping(&mut self) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "ping"}))?;
        Ok(())
    }

    // ========== State Queries ==========

    /// Get the current editor state.
    pub fn get_state(&mut self) -> Result<EditorState, EditorError> {
        let response = self.send_raw(json!({"action": "get_state"}))?;
        Self::parse_data(response)
    }

    /// Get all entities in the scene.
    pub fn get_entities(&mut self) -> Result<Vec<EntityInfo>, EditorError> {
        let response = self.send_raw(json!({"action": "get_entities"}))?;
        Self::parse_data(response)
    }

    /// Get detailed information about a specific entity.
    pub fn get_entity(&mut self, id: u64) -> Result<EntityDetails, EditorError> {
        let response = self.send_raw(json!({"action": "get_entity", "id": id}))?;
        Self::parse_data(response)
    }

    /// Dump full entity state including all components.
    pub fn dump_entity(&mut self, id: u64) -> Result<EntityDump, EditorError> {
        let response = self.send_raw(json!({"action": "dump_entity", "id": id}))?;
        Self::parse_data(response)
    }

    /// Get all components on an entity.
    pub fn get_entity_components(&mut self, id: u64) -> Result<Vec<ComponentInfo>, EditorError> {
        let response = self.send_raw(json!({"action": "get_entity_components", "id": id}))?;
        Self::parse_data(response)
    }

    // ========== Playback Control ==========

    /// Enter play mode.
    pub fn play(&mut self) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "play"}))?;
        Ok(())
    }

    /// Pause the game (while in play mode).
    pub fn pause(&mut self) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "pause"}))?;
        Ok(())
    }

    /// Resume the game after pause.
    pub fn resume(&mut self) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "resume"}))?;
        Ok(())
    }

    /// Stop playing and return to scene mode.
    pub fn stop(&mut self) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "stop"}))?;
        Ok(())
    }

    // ========== Entity Operations ==========

    /// Select an entity by ID.
    pub fn select_entity(&mut self, id: u64) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "select_entity", "id": id}))?;
        Ok(())
    }

    /// Create a new entity with the given name.
    pub fn create_entity(&mut self, name: &str) -> Result<u64, EditorError> {
        let response = self.send_raw(json!({"action": "create_entity", "name": name}))?;
        let created: CreatedEntity = Self::parse_data(response)?;
        Ok(created.id)
    }

    /// Delete an entity by ID.
    pub fn delete_entity(&mut self, id: u64) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "delete_entity", "id": id}))?;
        Ok(())
    }

    /// Set a property on an entity's component.
    pub fn set_property(
        &mut self,
        entity: u64,
        component: &str,
        field: &str,
        value: impl Into<serde_json::Value>,
    ) -> Result<(), EditorError> {
        self.send_raw(json!({
            "action": "set_property",
            "entity": entity,
            "component": component,
            "field": field,
            "value": value.into()
        }))?;
        Ok(())
    }

    // ========== Sprite Operations ==========

    /// Set the texture of an entity's sprite by asset ID.
    pub fn set_sprite_texture(&mut self, entity: u64, texture_id: u64) -> Result<(), EditorError> {
        self.set_property(entity, "Sprite", "texture", texture_id)
    }

    /// Set the size of an entity's sprite.
    pub fn set_sprite_size(&mut self, entity: u64, width: f32, height: f32) -> Result<(), EditorError> {
        self.set_property(entity, "Sprite", "size.x", width)?;
        self.set_property(entity, "Sprite", "size.y", height)
    }

    /// Set the flip state of an entity's sprite.
    pub fn set_sprite_flip(&mut self, entity: u64, flip_x: bool, flip_y: bool) -> Result<(), EditorError> {
        self.set_property(entity, "Sprite", "flip_x", flip_x)?;
        self.set_property(entity, "Sprite", "flip_y", flip_y)
    }

    /// Set the color of an entity's sprite (RGBA, values 0.0-1.0).
    pub fn set_sprite_color(&mut self, entity: u64, r: f32, g: f32, b: f32, a: f32) -> Result<(), EditorError> {
        self.set_property(entity, "Sprite", "color.r", r)?;
        self.set_property(entity, "Sprite", "color.g", g)?;
        self.set_property(entity, "Sprite", "color.b", b)?;
        self.set_property(entity, "Sprite", "color.a", a)
    }

    // ========== UI Control ==========

    /// Get the current UI state.
    pub fn get_ui_state(&mut self) -> Result<UiStateData, EditorError> {
        let response = self.send_raw(json!({"action": "get_ui_state"}))?;
        Self::parse_data(response)
    }

    /// List all panels.
    pub fn list_panels(&mut self) -> Result<Vec<PanelInfo>, EditorError> {
        let response = self.send_raw(json!({"action": "list_panels"}))?;
        Self::parse_data(response)
    }

    /// Get all clickable elements.
    pub fn get_clickable_elements(&mut self) -> Result<Vec<ClickableInfo>, EditorError> {
        let response = self.send_raw(json!({"action": "get_clickable_elements"}))?;
        Self::parse_data(response)
    }

    /// Focus a panel by ID.
    pub fn focus_panel(&mut self, panel: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "focus_panel", "panel": panel}))?;
        Ok(())
    }

    /// Click a UI element by ID.
    pub fn click_element(&mut self, id: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "click_element", "id": id}))?;
        Ok(())
    }

    /// Double-click a UI element by ID.
    pub fn double_click_element(&mut self, id: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "double_click_element", "id": id}))?;
        Ok(())
    }

    /// Right-click a UI element by ID.
    pub fn right_click_element(&mut self, id: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "right_click_element", "id": id}))?;
        Ok(())
    }

    /// Trigger a UI element by ID (simple click).
    pub fn trigger_element(&mut self, id: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "trigger_element", "id": id}))?;
        Ok(())
    }

    // ========== Scene Tree Control ==========

    /// Expand a tree node by path.
    pub fn expand_tree_node(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "expand_tree_node", "path": path}))?;
        Ok(())
    }

    /// Collapse a tree node by path.
    pub fn collapse_tree_node(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "collapse_tree_node", "path": path}))?;
        Ok(())
    }

    /// Select an entity by path in the scene tree.
    pub fn select_by_path(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "select_by_path", "path": path}))?;
        Ok(())
    }

    // ========== Asset Browser ==========

    /// Get the asset browser state.
    pub fn get_asset_browser_state(&mut self) -> Result<AssetBrowserData, EditorError> {
        let response = self.send_raw(json!({"action": "get_asset_browser_state"}))?;
        Self::parse_data(response)
    }

    /// Open an asset file.
    pub fn open_asset_file(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "open_asset_file", "path": path}))?;
        Ok(())
    }

    /// Select a file in the asset browser.
    pub fn select_asset_file(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "select_asset_file", "path": path}))?;
        Ok(())
    }

    /// Double-click on a file in the asset browser.
    pub fn double_click_asset_file(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "double_click_asset_file", "path": path}))?;
        Ok(())
    }

    /// Open a file in the editor from context menu.
    pub fn asset_context_open_in_editor(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "asset_context_open_in_editor", "path": path}))?;
        Ok(())
    }

    // ========== Script Editor ==========

    /// Open a script in the script editor.
    pub fn open_script(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "open_script", "path": path}))?;
        Ok(())
    }

    /// Save the current script.
    pub fn save_script(&mut self) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "save_script"}))?;
        Ok(())
    }

    /// Get the script editor state.
    pub fn get_script_editor_state(&mut self) -> Result<ScriptEditorData, EditorError> {
        let response = self.send_raw(json!({"action": "get_script_editor_state"}))?;
        Self::parse_data(response)
    }

    // ========== Project ==========

    /// Load a project from the specified path.
    pub fn load_project(&mut self, path: &str) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "load_project", "path": path}))?;
        Ok(())
    }

    // ========== Assets & Rendering ==========

    /// Get loaded assets info.
    pub fn get_assets(&mut self) -> Result<Vec<AssetInfo>, EditorError> {
        let response = self.send_raw(json!({"action": "get_assets"}))?;
        Self::parse_data(response)
    }

    /// Get renderer state.
    pub fn get_render_state(&mut self) -> Result<RenderStateData, EditorError> {
        let response = self.send_raw(json!({"action": "get_render_state"}))?;
        Self::parse_data(response)
    }

    /// Load a texture by asset ID.
    pub fn load_texture(&mut self, id: u64) -> Result<TextureLoadResult, EditorError> {
        let response = self.send_raw(json!({"action": "load_texture", "id": id}))?;
        Self::parse_data(response)
    }

    /// Load all registered textures.
    pub fn load_all_textures(&mut self) -> Result<Vec<TextureLoadResult>, EditorError> {
        let response = self.send_raw(json!({"action": "load_all_textures"}))?;
        Self::parse_data(response)
    }

    // ========== Testing Commands (require editor-side support) ==========

    /// Take a screenshot and save to the specified path.
    pub fn take_screenshot(&mut self, path: &str) -> Result<ScreenshotResult, EditorError> {
        let response = self.send_raw(json!({"action": "take_screenshot", "path": path}))?;
        Self::parse_data(response)
    }

    /// Get the last N log entries.
    pub fn get_log_tail(&mut self, lines: usize) -> Result<LogTailResult, EditorError> {
        let response = self.send_raw(json!({"action": "get_log_tail", "lines": lines}))?;
        Self::parse_data(response)
    }

    /// Wait for a number of frames to pass.
    pub fn wait_frames(&mut self, count: u32) -> Result<WaitFramesResult, EditorError> {
        let response = self.send_raw(json!({"action": "wait_frames", "count": count}))?;
        Self::parse_data(response)
    }

    // ========== Console ==========

    /// Toggle the console panel.
    pub fn toggle_console(&mut self) -> Result<(), EditorError> {
        self.send_raw(json!({"action": "toggle_console"}))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests require a running editor, so they're ignored by default.
    // Run with: cargo test -- --ignored

    #[test]
    #[ignore]
    fn test_connect_and_ping() {
        let mut client = EditorClient::connect_default().expect("Failed to connect");
        client.ping().expect("Ping failed");
    }

    #[test]
    #[ignore]
    fn test_get_state() {
        let mut client = EditorClient::connect_default().expect("Failed to connect");
        let state = client.get_state().expect("Failed to get state");
        println!("Mode: {}, Entities: {}", state.mode, state.entity_count);
    }
}
