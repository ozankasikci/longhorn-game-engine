//! Debug and render state command handlers.
//!
//! Handles render state queries and debugging utilities.

use longhorn_core::Sprite;
use longhorn_engine::Engine;
use longhorn_remote::{RemoteResponse, RenderStateData, ResponseData};

use crate::Editor;

// --- Render State Handlers ---

pub fn handle_get_render_state(_editor: &Editor, engine: &Engine) -> RemoteResponse {
    let (loaded_texture_count, texture_ids) = match engine.renderer() {
        Some(renderer) => {
            let ids = renderer.loaded_texture_ids();
            (ids.len(), ids.into_iter().map(|id| id.0).collect())
        }
        None => (0, Vec::new()),
    };

    let sprite_count = engine
        .world()
        .inner()
        .query::<&Sprite>()
        .iter()
        .count();

    let data = RenderStateData {
        loaded_texture_count,
        texture_ids,
        sprite_count,
    };

    RemoteResponse::with_data(ResponseData::RenderState(data))
}
