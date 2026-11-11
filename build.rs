use winresource;

fn main() {
  if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("src/icon.ico");
    // No manifest here: gpui's `windows-manifest` feature (enabled via gpui-component)
    // embeds one with Common-Controls v6 + PerMonitorV2 DPI awareness, which TaskDialogIndirect
    // and the UI need. Embedding a second manifest causes a "duplicate resource" link error.
    res.compile().unwrap();
  }
}
