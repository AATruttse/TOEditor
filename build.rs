fn main() {
    // Compile Slint UI with bundled translations for runtime language switch
    let config = slint_build::CompilerConfiguration::default()
        .with_bundled_translations("i18n");
    slint_build::compile_with_config("ui/main_window.slint", config).unwrap();

    println!("cargo:rerun-if-changed=ui/main_window.slint");
    println!("cargo:rerun-if-changed=ui/theme.slint");
    println!("cargo:rerun-if-changed=ui/dialogs.slint");
    println!("cargo:rerun-if-changed=ui/editors.slint");
    println!("cargo:rerun-if-changed=ui/context_menu.slint");
    println!("cargo:rerun-if-changed=ui/unit_table.slint");
    println!("cargo:rerun-if-changed=ui/org_chart.slint");
    println!("cargo:rerun-if-changed=i18n");
}
