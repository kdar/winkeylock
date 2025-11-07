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
  "blacklist": [
    {
      "key": 91,
      "shift": false,
      "ctrl": false,
      "alt": false,
      "win": false
    },
    {
      "key": 92,
      "shift": false,
      "ctrl": false,
      "alt": false,
      "win": false
    }
  ],
  "whitelist": [
    {
      "key": 83,
      "shift": true,
      "ctrl": false,
      "alt": false,
      "win": true
    },
    {
      "key": 68,
      "shift": false,
      "ctrl": false,
      "alt": false,
      "win": true
    }
  ]
}
```

## Key Codes

Common virtual key codes:
- `91`: Left Windows key
- `92`: Right Windows key
- `83`: S key
- `68`: D key
- `76`: L key
- `69`: E key
- `82`: R key

For a complete list of virtual key codes, see: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes

## Default Configuration

By default, the application:

**Blocks:**
- Windows key by itself (left and right)

**Allows:**
- Win + Shift + S (screenshot)
- Win + D (show desktop)
- Win + L (lock screen)
- Win + E (file explorer)
- Win + R (run dialog)

## How to Configure

1. Right-click the system tray icon
2. Select "Open configuration"
3. Edit the JSON file in Notepad
4. Save the file
5. Restart the application for changes to take effect

## Key Combination Format

Each key combination is defined with:
- `key`: Virtual key code (number)
- `shift`: Whether Shift key must be pressed (true/false)
- `ctrl`: Whether Ctrl key must be pressed (true/false)
- `alt`: Whether Alt key must be pressed (true/false)
- `win`: Whether Windows key must be pressed (true/false)

## Priority

Whitelist entries have higher priority than blacklist entries. If a key combination matches both a blacklist and whitelist entry, it will be allowed.

## Notes

- Configuration changes require restarting the application
- The application only blocks keys when Windows indicates the system is busy or in fullscreen mode
- Invalid configuration files will fall back to default settings