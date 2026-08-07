//! zts fork: format stdin as a .zts file and write the result to stdout.
//! Used when authoring spec files, and to check idempotence by hand.
use dprint_plugin_typescript::configuration::ConfigurationBuilder;
use dprint_plugin_typescript::{format_text, FormatTextOptions};
use std::io::Read;
use std::path::PathBuf;

fn main() {
  let mut text = String::new();
  std::io::stdin().read_to_string(&mut text).unwrap();
  let indent_width = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(2);
  let config = ConfigurationBuilder::new().indent_width(indent_width).build();
  match format_text(FormatTextOptions {
    path: &PathBuf::from("file.zts"),
    extension: None,
    text: text.clone(),
    config: &config,
    external_formatter: None,
  }) {
    Ok(Some(out)) => print!("{out}"),
    Ok(None) => print!("{text}"),
    Err(e) => {
      eprintln!("ERROR: {e}");
      std::process::exit(1);
    }
  }
}
