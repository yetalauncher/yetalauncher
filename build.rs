use std::{path::PathBuf, str::FromStr};

use slint_build::CompilerConfiguration;

fn main() {
    let config = CompilerConfiguration::new()
    .with_style(String::from("fluent-dark"))
    .with_include_paths(vec![
        PathBuf::from_str("reources/default_instance.png").unwrap()
    ]);

    slint_build::compile_with_config("gui/window.slint", config).unwrap();
}