# WinKeyLock Configuration

WinKeyLock now supports configurable blacklist and whitelist functionality for keyboard shortcuts.

## Configuration File Location

The configuration file is automatically created at:

- Windows: `%APPDATA%\winkeylock\config.json`

## Configuration Format

The configuration file uses JSON format with two main sections:

### Blacklist

Key combinations that should be blocked when the application is busy or in fullscreen mode.

### Whitelist

Key combinations that should be explicitly allowed, even if they match a blacklist entry.

## Example Configuration

```json
{
  "blacklist": ["lwin"],
  "whitelist": []
}
```

## Key Combination Format

Key combinations are specified as strings using the format: `modifier+...+key`

### Supported Modifiers

- `ctrl` or `control`: Control key
- `alt`: Alt key
- `shift`: Shift key
- `lwin`, `rwin`, or `super`: Windows key

### Supported Keys

**Letters**: `a` through `z`
**Numbers**: `0` through `9`
**Function Keys**: `f1` through `f24`

**Special Keys**:

- `space`, `enter`, `return`, `tab`, `escape`, `esc`
- `backspace`, `delete`, `del`, `insert`, `ins`
- `home`, `end`, `pageup`, `pagedown`
- `up`, `down`, `left`, `right`
- `printscreen`, `prtsc`, `pause`
- `capslock`, `numlock`, `scrolllock`

**Punctuation**:

- `semicolon`, `equals`, `comma`, `minus`, `period`, `slash`
- `grave`, `leftbracket`, `backslash`, `rightbracket`, `quote`

**Examples**:

- `lwin`: Windows key alone
- `lwin+shift+s`: Windows + Shift + S (screenshot)
- `ctrl+alt+delete`: Ctrl + Alt + Delete
- `alt+f4`: Alt + F4

## Default Configuration

By default, the application:

**Blocks:**

- Windows key by itself (left and right)

## How to Configure

1. Right-click the system tray icon
2. Select "Open configuration"
3. Edit the JSON file in your editor using the string format (e.g., "win+shift+s")
4. Save the file
5. Restart the application for changes to take effect

## Priority

Whitelist entries have higher priority than blacklist entries. If a key combination matches both a blacklist and whitelist entry, it will be allowed.

## Notes

- Configuration changes require restarting the application
- The application only blocks keys when Windows indicates the system is busy or in fullscreen mode (like in gaming)
- Invalid configuration files will fall back to default settings
