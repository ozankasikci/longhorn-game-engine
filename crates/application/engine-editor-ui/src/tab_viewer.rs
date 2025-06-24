// Tab viewer implementation for the docking system

use crate::types::ConsoleMessage;
use crate::types::PanelType;
use eframe::egui;
use egui_dock::{NodeIndex, SurfaceIndex, TabViewer};
/// Trait for the editor application
pub trait EditorApp {
    fn show_panel(&mut self, ui: &mut egui::Ui, panel_type: PanelType);
}

/// Wrapper to avoid borrowing conflicts when using TabViewer
pub struct EditorTabViewer<'a> {
    pub editor: &'a mut dyn EditorApp,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = PanelType;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelType::Hierarchy => "Hierarchy".into(),
            PanelType::Inspector => "Inspector".into(),
            PanelType::SceneView => "Scene".into(),
            PanelType::GameView => "Game".into(),
            PanelType::Console => "Console".into(),
            PanelType::Project => "Project".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        self.editor.show_panel(ui, *tab);
    }

    fn context_menu(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        if ui.button("Close Tab").clicked() {
            // Panel closed
            ui.close_menu();
        }
        if ui.button("Duplicate Tab").clicked() {
            // Note: We can't modify dock_state here since it's already borrowed
            // Panel duplicated
            ui.close_menu();
        }
    }
}
