use dprint_plugin_typescript::configuration::ConfigurationBuilder;
use dprint_plugin_typescript::{format_text, FormatTextOptions};
use std::path::PathBuf;
use std::time::Instant;

fn try_fmt(name: &str, src: &str) {
  let config = ConfigurationBuilder::new().build();
  let t = Instant::now();
  let res = std::panic::catch_unwind(|| {
    format_text(FormatTextOptions {
      path: &PathBuf::from("probe.ts"),
      extension: Some("ts"),
      text: src.to_string(),
      config: &config,
      external_formatter: None,
    })
  });
  let ms = t.elapsed().as_millis();
  match res {
    Ok(Ok(Some(out))) => println!("{name:32} {ms:>6}ms OK(changed)   {:?}", out),
    Ok(Ok(None)) => println!("{name:32} {ms:>6}ms OK(unchanged)"),
    Ok(Err(e)) => println!("{name:32} {ms:>6}ms ERR {}", e.to_string().lines().next().unwrap_or("")),
    Err(_) => println!("{name:32} {ms:>6}ms PANIC"),
  }
}

fn main() {
  std::panic::set_hook(Box::new(|info| {
    if let Some(m) = info.payload().downcast_ref::<String>() {
      eprintln!("[panic] {m}");
    }
  }));
  let args: Vec<String> = std::env::args().skip(1).collect();
  if !args.is_empty() {
    for (i, a) in args.iter().enumerate() {
      try_fmt(&format!("arg{i}"), a);
    }
    return;
  }
  try_fmt("enum", "enum Shape {\n  Circle { r: number },\n  Square { side: number },\n}\n");
  try_fmt("enum mut/empty", "enum Counter {\n  Cell { mut count: number, label: string },\n  Empty {},\n}\n");
  try_fmt("match", "declare const shape: Shape;\nconst area = match (shape) {\n  Circle { radius } => PI * radius ** 2,\n  Square { side } => side ** 2,\n};\n");
  try_fmt("match lit/wildcard", "const r = match (l) {\n  'debug' => 0,\n  -1 => 1,\n  _ => 2,\n};\n");
  try_fmt("match block arm", "const a = match (s) {\n  Circle { radius } => {\n    const r2 = radius * radius;\n    3.14 * r2\n  },\n  Square { side } => side ** 2,\n};\n");
  try_fmt("if expr", "const a = if (b === 0) { 3 } else { 4 };\n");
  try_fmt("if expr chain", "const g = if (b > 90) { \"A\" } else if (b > 80) { \"B\" } else { \"C\" };\n");
  try_fmt("if expr multi", "const r = if (n > 0) {\n  const cached = expensive(n);\n  cached * 2\n} else {\n  0\n};\n");
  try_fmt("newtype", "newtype AccountId = string;\n");
  try_fmt("union", "union DeleteOutcome = 'soft' | 'hard' | 'unknown';\n");
  try_fmt("impl", "impl Display for Shape {\n  fmt(self): string {\n    return \"x\";\n  }\n}\n");
  try_fmt("impl multi/assoc", "impl From<string>, From<number> for Id {\n  from(value: string | number): Id {\n    return Id.Str(\"a\");\n  }\n}\n");
  try_fmt("not", "const a = not ready;\nconst b = not not ready;\nconst e = not (items.length > 0);\n");
  try_fmt("try", "function f(): Result<number, string> {\n  const once = divide(n, 2)?;\n  return Ok(once);\n}\n");
  try_fmt("nonempty array", "declare const xs: number[+];\n");
  try_fmt("constrict", "constrict A == B;\nconstrict C != D;\nconstrict E extends F;\n");
}
