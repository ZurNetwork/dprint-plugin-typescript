//! zts fork: `fmt(fmt(x)) == fmt(x)` over the whole zts spec corpus.
//!
//! The spec harness already formats twice (`RunSpecsOptions::format_twice`),
//! but it checks the second pass against the spec's own `[expect]` block. That
//! makes idempotence a *consequence* of the expectations being right rather
//! than a property in its own right, and it says nothing about a spec whose
//! expectation someone updated by hand.
//!
//! This asserts the property directly, on every zts spec input AND on every
//! expected output, at two indent widths and both semicolon settings — so a
//! rule that is stable at one width but oscillates at another is caught.
//!
//! Not a substitute for the spec suite: this only says the formatter reaches a
//! fixed point, not that the fixed point is the right text. The specs say that.

use dprint_plugin_typescript::configuration::{Configuration, ConfigurationBuilder, SemiColons};
use dprint_plugin_typescript::{format_text, FormatTextOptions};
use std::path::{Path, PathBuf};

/// One `== message ==` block of a spec file.
struct Spec {
  file: String,
  message: String,
  file_name: String,
  input: String,
  expected: String,
}

fn collect_specs(dir: &Path, out: &mut Vec<Spec>) {
  let mut entries: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
  entries.sort();
  for path in entries {
    if path.is_dir() {
      collect_specs(&path, out);
      continue;
    }
    let text = std::fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
    let (file_name, text) = match text.strip_prefix("--") {
      Some(rest) => {
        let end = rest.find("--\n").expect("unterminated file name header");
        (rest[..end].trim().to_string(), rest[end + 3..].to_string())
      }
      None => ("file.ts".to_string(), text),
    };
    // a `~~ config ~~` header changes the expected output but not whether the
    // formatter settles, so the block is stripped and the input still checked
    let text = match text.strip_prefix("~~") {
      Some(rest) => rest[rest.find("~~\n").expect("unterminated config header") + 3..].to_string(),
      None => text,
    };

    let lines: Vec<&str> = text.split('\n').collect();
    let starts: Vec<usize> = lines.iter().enumerate().filter(|(_, l)| l.starts_with("==")).map(|(i, _)| i).collect();
    for (n, &start) in starts.iter().enumerate() {
      let end = starts.get(n + 1).copied().unwrap_or(lines.len());
      let message_line = lines[start];
      if message_line.to_ascii_lowercase().contains("(skip)") {
        continue;
      }
      let body = lines[start + 1..end].join("\n");
      let Some((input, expected)) = body.split_once("[expect]") else {
        panic!("spec block without [expect] in {}", path.display());
      };
      out.push(Spec {
        file: path.display().to_string(),
        message: message_line.trim_matches(['=', ' ']).to_string(),
        file_name: file_name.clone(),
        input: input[..input.len().saturating_sub(1)].to_string(),
        expected: expected.strip_prefix('\n').unwrap_or(expected).to_string(),
      });
    }
  }
}

fn format(file_name: &str, text: &str, config: &Configuration) -> String {
  match format_text(FormatTextOptions {
    path: &PathBuf::from(file_name),
    extension: None,
    text: text.to_string(),
    config,
    external_formatter: None,
  }) {
    Ok(Some(output)) => output,
    Ok(None) => text.to_string(),
    Err(err) => panic!("failed to format:\n{text}\n\n{err}"),
  }
}

#[test]
fn zts_specs_are_idempotent() {
  let mut specs = Vec::new();
  collect_specs(Path::new("./tests/specs/zts"), &mut specs);
  assert!(specs.len() > 50, "expected the zts spec corpus, found {} specs", specs.len());

  let configs = [
    ("indent 2", ConfigurationBuilder::new().indent_width(2).build()),
    ("indent 4", ConfigurationBuilder::new().indent_width(4).build()),
    ("no semicolons", ConfigurationBuilder::new().semi_colons(SemiColons::Asi).build()),
    ("narrow", ConfigurationBuilder::new().line_width(40).build()),
  ];

  let mut checked = 0usize;
  for spec in &specs {
    for (config_name, config) in &configs {
      // start from the input and from the expectation: both must be fixed
      // points after one pass, whichever the caller happens to hand us
      for start in [&spec.input, &spec.expected] {
        let once = format(&spec.file_name, start, config);
        let twice = format(&spec.file_name, &once, config);
        assert_eq!(
          once, twice,
          "not idempotent [{config_name}] in {} :: {}\n--- input ---\n{start}\n--- once ---\n{once}\n--- twice ---\n{twice}",
          spec.file, spec.message,
        );
        // a third pass, because a two-cycle oscillation would pass the above
        let thrice = format(&spec.file_name, &twice, config);
        assert_eq!(twice, thrice, "not idempotent on the third pass [{config_name}] in {} :: {}", spec.file, spec.message);
        checked += 1;
      }
    }
  }

  eprintln!("zts idempotence: {} specs x {} configs x 2 starts = {} fixed points", specs.len(), configs.len(), checked);
}
