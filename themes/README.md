# Bundled GPUI Kit themes

These JSON files are unmodified copies from
[longbridge/gpui-kit v0.9.1](https://github.com/longbridge/gpui-kit/tree/v0.9.1/themes),
matching the application's GPUI Kit dependency. They are embedded in the executable;
users do not need to download them or keep this directory next to the app.

`build.rs` automatically includes every `.json` file directly in this directory,
in filename order. Add or remove a theme file and rebuild; no Rust file list needs
updating. Other files, including this README and the license, are ignored.

Upstream copyright and license terms are in [LICENSE-APACHE](LICENSE-APACHE).
Individual theme author and source metadata remain in each JSON file.

WinKeyLock's **Default** palette is defined separately in
`src/config_ui/gpui_kit_ui/theme.rs` to preserve the original appearance.
GPUI Kit's **Default Light** and **Default Dark** are separate choices.
