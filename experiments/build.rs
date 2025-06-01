fn main() {
    slint_build::compile("ui/unity_editor.slint").unwrap();
    slint_build::compile("ui/simple_dockable_editor.slint").unwrap();
}