use winres;

fn main() {
  if cfg!(target_os = "windows") {
    let mut res = winres::WindowsResource::new();
    res.set_icon("src/icon.ico");
    res.compile().unwrap();
  }
}
