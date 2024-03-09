use slint_build::CompilerConfiguration;

fn main() {
    let config = CompilerConfiguration::new()
    .with_style(String::from("fluent-dark"));

    slint_build::compile_with_config("gui/window.slint", config).unwrap();
}