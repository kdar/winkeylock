use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
  bundle_themes();
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=Cargo.toml");
  println!("cargo:rerun-if-changed=src/icon.ico");

  if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("src/icon.ico");
    // No manifest here: gpui's `windows-manifest` feature (enabled via gpui-component)
    // embeds one with Common-Controls v6 + PerMonitorV2 DPI awareness, which TaskDialogIndirect
    // and the UI need. Embedding a second manifest causes a "duplicate resource" link error.
    res.compile().unwrap();
  }
}

/// Generate include_str! calls so every theme is embedded in the executable.
fn bundle_themes() {
  let themes_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("themes");
  // Watching the directory also catches additions, removals, and renames.
  println!("cargo:rerun-if-changed={}", themes_dir.display());
  let mut paths: Vec<_> = fs::read_dir(&themes_dir)
    .expect("cannot read themes directory")
    .map(|entry| entry.expect("cannot read theme directory entry").path())
    .filter(|path| {
      path.is_file()
        && path
          .extension()
          .is_some_and(|extension| extension == "json")
    })
    .collect();
  paths.sort();

  let mut source = String::from("const BUNDLED_THEMES: &[&str] = &[\n");
  for path in paths {
    println!("cargo:rerun-if-changed={}", path.display());
    // Rust's Debug string formatting quotes and escapes Windows paths correctly.
    let path = path.to_str().expect("theme path must be valid UTF-8");
    source.push_str(&format!("  include_str!({path:?}),\n"));
  }
  source.push_str("];\n");
  let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bundled_themes.rs");
  fs::write(output, source).expect("cannot write bundled theme list");
}
