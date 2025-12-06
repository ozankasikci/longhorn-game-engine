use egui::Ui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer, Style};

use crate::styling::{Colors, Radius};

/// Different types of dockable panels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelType {
    Hierarchy,
    Inspector,
    SceneView,
    GameView,
    Console,
    Project,
    ScriptEditor,
}

impl PanelType {
    pub fn title(&self) -> &'static str {
        match self {
            PanelType::Hierarchy => "Hierarchy",
            PanelType::Inspector => "Inspector",
            PanelType::SceneView => "Scene",
            PanelType::GameView => "Game",
            PanelType::Console => "Console",
            PanelType::Project => "Project",
            PanelType::ScriptEditor => "Script Editor",
        }
    }
}

/// Trait for rendering panels - implemented by Editor
pub trait PanelRenderer {
    fn show_panel(&mut self, ui: &mut Ui, panel_type: PanelType);
}

/// TabViewer implementation for the dock system
pub struct EditorTabViewer<'a, T: PanelRenderer> {
    pub editor: &'a mut T,
}

impl<'a, T: PanelRenderer> TabViewer for EditorTabViewer<'a, T> {
    type Tab = PanelType;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        self.editor.show_panel(ui, *tab);
    }
}

/// Create the default dock layout matching v1
pub fn create_default_dock_state() -> DockState<PanelType> {
    // Start with Scene and Game views tabbed in the center
    let mut dock_state = DockState::new(vec![
        PanelType::SceneView,
        PanelType::GameView,
    ]);

    // Add Hierarchy to the left (20% width)
    let [_main, _left] = dock_state.main_surface_mut().split_left(
        NodeIndex::root(),
        0.2,
        vec![PanelType::Hierarchy],
    );

    // Add Inspector to the right (80% of remaining = rightmost 20%)
    let [_main, _right] = dock_state.main_surface_mut().split_right(
        NodeIndex::root(),
        0.8,
        vec![PanelType::Inspector],
    );

    // Add Console and Project tabbed at the bottom (70% for main, 30% for bottom)
    let [_main, _bottom] = dock_state.main_surface_mut().split_below(
        NodeIndex::root(),
        0.7,
        vec![PanelType::Console, PanelType::Project],
    );

    dock_state
}

/// Create dock styling that matches the Longhorn theme
pub fn create_dock_style(ui: &Ui) -> Style {
    let mut style = Style::from_egui(ui.style());

    // Tab styling - use vibrant accent for active tabs
    style.tab.active.bg_fill = Colors::ACCENT;
    style.tab.active.text_color = Colors::TEXT_ON_ACCENT;
    style.tab.focused.text_color = Colors::TEXT_ON_ACCENT;
    style.tab.focused.bg_fill = Colors::ACCENT;

    // Inactive tabs - darker background
    style.tab.inactive.bg_fill = Colors::BG_WINDOW;
    style.tab.inactive.text_color = Colors::TEXT_SECONDARY;

    // Hovered tabs
    style.tab.hovered.bg_fill = Colors::BG_WIDGET_HOVERED;
    style.tab.hovered.text_color = Colors::TEXT_ON_ACCENT;

    // Tab outline and rounding
    style.tab.active.outline_color = Colors::ACCENT;
    style.tab.active.rounding = Radius::all(Radius::MEDIUM);
    style.tab.inactive.rounding = Radius::all(Radius::MEDIUM);
    style.tab.inactive.outline_color = Colors::STROKE_DEFAULT;

    // Separator styling
    style.separator.width = 1.0;
    style.separator.color_idle = Colors::STROKE_DEFAULT;
    style.separator.color_hovered = Colors::ACCENT;
    style.separator.color_dragged = Colors::ACCENT;

    style
}

/// Show the dock area with the given state
pub fn show_dock_area<T: PanelRenderer>(
    ui: &mut Ui,
    dock_state: &mut DockState<PanelType>,
    editor: &mut T,
) {
    let style = create_dock_style(ui);

    DockArea::new(dock_state)
        .style(style)
        .show_inside(ui, &mut EditorTabViewer { editor });
}
