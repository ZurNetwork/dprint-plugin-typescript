//! zts fork: sweep every spec input through the formatter WITHOUT the
//! dprint-development harness, which panics (and, in parallel mode, deadlocks)
//! on the first spec that fails to parse. This reports every failure in one run,
//! and with `--emit <dir>` writes each spec's formatted output to a file so the
//! same sweep can be run against stock dprint-plugin-typescript and diffed.
//!
//! EVERY spec input is formatted so that no parse failure can hide, but only
//! default-config specs are emitted for the parity diff — a spec carrying a
//! `~~ config ~~` header would format differently under the default config, and
//! comparing those outputs would prove nothing. `(skip)`, `(only)` and `(trace)`
//! specs are left alone.

use dprint_plugin_typescript::configuration::ConfigurationBuilder;
use dprint_plugin_typescript::{format_text, FormatTextOptions};
use std::path::{Path, PathBuf};

struct Spec {
  file: PathBuf,
  file_name: String,
  message: String,
  input: String,
  default_config: bool,
}

fn collect(dir: &Path, out: &mut Vec<Spec>) {
  let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
  entries.sort();
  for path in entries {
    if path.is_dir() {
      collect(&path, out);
      continue;
    }
    let text = std::fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
    // `-- file.tsx --` header overrides the file name
    let (file_name, text) = if let Some(rest) = text.strip_prefix("--") {
      let end = rest.find("--\n").expect("no closing --");
      (rest[..end].trim().to_string(), rest[end + 3..].to_string())
    } else {
      ("file.ts".to_string(), text)
    };
    // `~~ config ~~` header: still formatted (a parse failure is a parse failure
    // whatever the config), but not emitted for the parity diff.
    let (default_config, text) = if let Some(rest) = text.strip_prefix("~~") {
      let end = rest.find("~~\n").expect("no closing ~~");
      (false, rest[end + 3..].to_string())
    } else {
      (true, text)
    };
    let lines: Vec<&str> = text.split('\n').collect();
    let starts: Vec<usize> = lines.iter().enumerate().filter(|(_, l)| l.starts_with("==")).map(|(i, _)| i).collect();
    for (n, &start) in starts.iter().enumerate() {
      let end = starts.get(n + 1).copied().unwrap_or(lines.len());
      let message_line = lines[start];
      let lower = message_line.to_ascii_lowercase();
      if lower.contains("(skip)") || lower.contains("(only)") || lower.contains("(trace)") {
        continue;
      }
      let body = lines[start + 1..end].join("\n");
      let Some((input, _expected)) = body.split_once("[expect]") else { continue };
      out.push(Spec {
        file: path.clone(),
        file_name: file_name.clone(),
        message: message_line.trim_matches(['=', ' ']).to_string(),
        input: input[..input.len().saturating_sub(1)].to_string(),
        default_config,
      });
    }
  }
}

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let emit_dir = args.windows(2).find(|w| w[0] == "--emit").map(|w| PathBuf::from(&w[1]));

  let mut specs = Vec::new();
  collect(Path::new("./tests/specs"), &mut specs);

  let config = ConfigurationBuilder::new().build();
  let mut ok = 0usize;
  let mut failed = Vec::new();
  std::panic::set_hook(Box::new(|_| {}));

  for spec in &specs {
    let result = std::panic::catch_unwind(|| {
      format_text(FormatTextOptions {
        path: &PathBuf::from(&spec.file_name),
        extension: None,
        text: spec.input.clone(),
        config: &config,
        external_formatter: None,
      })
    });
    let output = match result {
      Ok(Ok(Some(text))) => Ok(text),
      Ok(Ok(None)) => Ok(spec.input.clone()),
      Ok(Err(err)) => Err(format!("ERR {}", err.to_string().lines().next().unwrap_or(""))),
      Err(_) => Err("PANIC".to_string()),
    };
    match output {
      Ok(text) => {
        ok += 1;
        if let (Some(dir), true) = (&emit_dir, spec.default_config) {
          let rel = spec.file.strip_prefix("./tests/specs").unwrap();
          let name = format!("{}__{}", rel.to_string_lossy().replace(['/', '\\'], "_"), sanitize(&spec.message));
          std::fs::create_dir_all(dir).unwrap();
          std::fs::write(dir.join(name), text).unwrap();
        }
      }
      Err(why) => failed.push((spec, why)),
    }
  }

  let _ = std::panic::take_hook();
  for (spec, why) in &failed {
    println!("{}\n  {}\n  {}", spec.file.display(), spec.message, why);
  }
  println!("\n{} specs formatted, {} failed", ok, failed.len());
  if !failed.is_empty() {
    std::process::exit(1);
  }
}

fn sanitize(s: &str) -> String {
  s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}
