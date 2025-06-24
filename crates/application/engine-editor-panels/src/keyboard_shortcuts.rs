use eframe::egui;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    Undo,
    Redo,
    Delete,
    Rename,
    Copy,
    Paste,
    Cut,
    SelectAll,
    Search,
    NewFolder,
    Refresh,
}

#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub key: egui::Key,
    pub modifiers: egui::Modifiers,
}

impl KeyboardShortcut {
    pub fn new(key: egui::Key, modifiers: egui::Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn matches(&self, input: &egui::InputState) -> bool {
        input.key_pressed(self.key) && input.modifiers == self.modifiers
    }
}

pub struct ShortcutManager {
    shortcuts: HashMap<ShortcutAction, KeyboardShortcut>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();

        // Standard shortcuts
        shortcuts.insert(
            ShortcutAction::Undo,
            KeyboardShortcut::new(egui::Key::Z, egui::Modifiers::COMMAND),
        );

        shortcuts.insert(
            ShortcutAction::Redo,
            KeyboardShortcut::new(
                egui::Key::Z,
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            ),
        );

        shortcuts.insert(
            ShortcutAction::Delete,
            KeyboardShortcut::new(egui::Key::Delete, egui::Modifiers::NONE),
        );

        shortcuts.insert(
            ShortcutAction::Rename,
            KeyboardShortcut::new(egui::Key::F2, egui::Modifiers::NONE),
        );

        shortcuts.insert(
            ShortcutAction::Copy,
            KeyboardShortcut::new(egui::Key::C, egui::Modifiers::COMMAND),
        );

        shortcuts.insert(
            ShortcutAction::Paste,
            KeyboardShortcut::new(egui::Key::V, egui::Modifiers::COMMAND),
        );

        shortcuts.insert(
            ShortcutAction::Cut,
            KeyboardShortcut::new(egui::Key::X, egui::Modifiers::COMMAND),
        );

        shortcuts.insert(
            ShortcutAction::SelectAll,
            KeyboardShortcut::new(egui::Key::A, egui::Modifiers::COMMAND),
        );

        shortcuts.insert(
            ShortcutAction::Search,
            KeyboardShortcut::new(egui::Key::F, egui::Modifiers::COMMAND),
        );

        shortcuts.insert(
            ShortcutAction::NewFolder,
            KeyboardShortcut::new(
                egui::Key::N,
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            ),
        );

        shortcuts.insert(
            ShortcutAction::Refresh,
            KeyboardShortcut::new(egui::Key::F5, egui::Modifiers::NONE),
        );

        Self { shortcuts }
    }
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_shortcuts(&self, ctx: &egui::Context) -> Vec<ShortcutAction> {
        let mut triggered = Vec::new();

        ctx.input(|input| {
            for (action, shortcut) in &self.shortcuts {
                if shortcut.matches(input) {
                    triggered.push(*action);
                }
            }
        });

        triggered
    }

    pub fn get_shortcut(&self, action: ShortcutAction) -> Option<&KeyboardShortcut> {
        self.shortcuts.get(&action)
    }

    pub fn format_shortcut(&self, action: ShortcutAction) -> String {
        if let Some(shortcut) = self.get_shortcut(action) {
            let mut parts = Vec::new();

            if shortcut.modifiers.ctrl || shortcut.modifiers.command {
                parts.push(if cfg!(target_os = "macos") {
                    "⌘"
                } else {
                    "Ctrl"
                });
            }
            if shortcut.modifiers.shift {
                parts.push("Shift");
            }
            if shortcut.modifiers.alt {
                parts.push("Alt");
            }

            parts.push(match shortcut.key {
                egui::Key::Delete => "Delete",
                egui::Key::F2 => "F2",
                egui::Key::F5 => "F5",
                egui::Key::A => "A",
                egui::Key::C => "C",
                egui::Key::F => "F",
                egui::Key::N => "N",
                egui::Key::V => "V",
                egui::Key::X => "X",
                egui::Key::Z => "Z",
                _ => "?",
            });

            parts.join("+")
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input(key: egui::Key, modifiers: egui::Modifiers) -> egui::InputState {
        let mut input = egui::InputState::default();
        input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            modifiers,
            repeat: false,
        });
        input.modifiers = modifiers;
        input
    }

    #[test]
    fn test_shortcut_matching() {
        let shortcut = KeyboardShortcut::new(egui::Key::Z, egui::Modifiers::COMMAND);

        let matching_input = create_test_input(egui::Key::Z, egui::Modifiers::COMMAND);
        assert!(shortcut.matches(&matching_input));

        let wrong_key = create_test_input(egui::Key::X, egui::Modifiers::COMMAND);
        assert!(!shortcut.matches(&wrong_key));

        let wrong_modifiers = create_test_input(egui::Key::Z, egui::Modifiers::NONE);
        assert!(!shortcut.matches(&wrong_modifiers));
    }

    #[test]
    fn test_default_shortcuts() {
        let manager = ShortcutManager::default();

        // Test that all default shortcuts are registered
        assert!(manager.get_shortcut(ShortcutAction::Undo).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Redo).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Delete).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Rename).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Copy).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Paste).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Cut).is_some());
        assert!(manager.get_shortcut(ShortcutAction::SelectAll).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Search).is_some());
        assert!(manager.get_shortcut(ShortcutAction::NewFolder).is_some());
        assert!(manager.get_shortcut(ShortcutAction::Refresh).is_some());
    }

    #[test]
    fn test_format_shortcut() {
        let manager = ShortcutManager::default();

        // Test formatting for different platforms
        if cfg!(target_os = "macos") {
            assert_eq!(manager.format_shortcut(ShortcutAction::Undo), "⌘+Z");
            assert_eq!(manager.format_shortcut(ShortcutAction::Redo), "⌘+Shift+Z");
        } else {
            assert_eq!(manager.format_shortcut(ShortcutAction::Undo), "Ctrl+Z");
            assert_eq!(
                manager.format_shortcut(ShortcutAction::Redo),
                "Ctrl+Shift+Z"
            );
        }

        assert_eq!(manager.format_shortcut(ShortcutAction::Delete), "Delete");
        assert_eq!(manager.format_shortcut(ShortcutAction::Rename), "F2");
    }

    #[test]
    fn test_undo_redo_shortcuts() {
        let undo_shortcut = KeyboardShortcut::new(egui::Key::Z, egui::Modifiers::COMMAND);
        let redo_shortcut = KeyboardShortcut::new(
            egui::Key::Z,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        );

        let undo_input = create_test_input(egui::Key::Z, egui::Modifiers::COMMAND);
        assert!(undo_shortcut.matches(&undo_input));
        assert!(!redo_shortcut.matches(&undo_input));

        let redo_input = create_test_input(
            egui::Key::Z,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        );
        assert!(!undo_shortcut.matches(&redo_input));
        assert!(redo_shortcut.matches(&redo_input));
    }
}
