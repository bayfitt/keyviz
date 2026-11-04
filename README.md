# Sticky Modifier Indicator

A Windows adaptation of Keyviz (upstream commit 8225e4157ed170a56d37366be24573f614b9ce06) that displays Windows Sticky Keys modifier state in the primary display's lower-right work area. Shift, Ctrl, Alt and Windows are shown; a brighter symbol indicates a locked modifier. The utility reads Windows Sticky Keys state without changing it. The window is hidden whenever no modifier is latched or locked. Use the notification-area icon to stop, start, or quit it.

This is a standalone Windows-specific fork, not the general-purpose Keyviz keystroke and mouse visualizer. It is derived from [Keyviz](https://github.com/mulaRahul/keyviz), licensed under GPL-3.0; see [LICENSE](LICENSE) and the [upstream README](README-UPSTREAM.md). It is not affiliated with Keyviz or Apple.

Enable Sticky Keys in **Settings → Accessibility → Keyboard** to use the indicator. The appearance is inspired by macOS; Windows controls the actual Sticky Keys behavior.

The Windows ARM64 installer is available on the [releases page](https://github.com/bayfitt/keyviz/releases). It is signed with Ben Fitterman's self-signed certificate, which is not trusted by Windows by default, so Windows may warn or block it. The signature has no trusted timestamp.

To build, install Node.js, Rust, Microsoft C++ Build Tools and the WebView2 prerequisites, then run `npm ci` and `npm run tauri -- build --no-bundle`. The current build has been tested on Windows 11 ARM64; other configurations have not been verified.
