# Xtunnel Games

Split tunneling client for games and apps via Aether.

## Features

- **Serverless tunnel**: Aether (MASQUE/WireGuard/gool)
- **Multiple providers**: V2Ray/Xray, WireGuard, OpenVPN, SOCKS5/HTTP proxy
- **Per-app tunneling**: PID-based WinDivert routing
- **Gaming optimized**: UDP support, gVisor TCP stack, low latency

## Development

```bash
# Install dependencies
npm install

# Run dev server
npm run tauri dev

# Build for production
npm run tauri build
```

## Architecture

- **Frontend**: React 18 + TypeScript + Tailwind CSS + Vazirmatn font
- **Backend**: Rust + Tauri 2
- **Tunneling**: Aether (serverless) + sing-box (TUN bridge) + WinDivert (PID routing)
- **Binaries bundled**: Aether, sing-box, Xray, wintun, WinDivert

## Build

Requires Windows 10/11 x64. Run as Administrator for WinDivert driver installation.

```bash
npm run tauri build
```

Output: `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/xtunnel-games-<version>-x64.exe`

## License

GPL-3.0-or-later