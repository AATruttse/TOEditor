fn main() {
    // Only compile the main window - other components are imported
    slint_build::compile("ui/main_window.slint").unwrap();
}
