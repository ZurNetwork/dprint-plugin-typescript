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
  try_fmt("vanilla ts", "const   t=5;\nfunction  foo( a:number,b:string ):void{ if(a){console.log(b)} }\n");
  try_fmt("ts enum (zts hard error)", "enum E { A, B }\n");
  try_fmt("zts enum-with-data", "enum Shape {\n  Circle { r: number },\n  Square { s: number },\n}\n");
  try_fmt("zts match expr", "declare const shape: Shape;\nconst area = match (shape) {\n  Circle { radius } => radius,\n};\n");
  try_fmt("zts newtype", "newtype UserId = string;\n");
  try_fmt("zts union", "union Level = 'a' | 'b';\n");
  try_fmt("zts expression if", "const  y = if (a) { 1 } else { 2 };\n");
  try_fmt("zts not", "const  z = not  a;\n");
  try_fmt("zts impl block", "impl Display for Status {\n  fmt(self): string { return 'x'; }\n}\n");
  try_fmt("zts try postfix", "function f(): Result<number, string> { const q = foo()?; return Ok(q); }\n");
  try_fmt("zts non-empty array", "declare const xs: number[+];\n");
}
