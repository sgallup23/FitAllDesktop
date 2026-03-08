# FitAll Desktop — Setup Guide

Cross-platform desktop wrapper for FitAll Fitness Platform.
Built with **Tauri v2** (Rust + native webview). ~10MB installer.

## Supported Platforms

| Platform | Format | Webview |
|----------|--------|---------|
| Windows 10/11 | `.msi`, `.exe` (NSIS) | WebView2 (Edge) |
| macOS 12+ | `.dmg` | WebKit (Safari) |
| Linux (Ubuntu/Debian) | `.deb`, `.AppImage` | WebKitGTK |

## Features

- Loads fitall.auraalpha.cc in a native window (instant updates, no reinstall needed)
- System tray with fitness quick-access (Dashboard, Quick Log, Workouts, Nutrition)
- Native desktop notifications for workout reminders and achievements
- Auto-update via GitHub Releases
- Minimize to tray (keeps running in background)
- Window state persistence (remembers size/position)
- Offline detection with retry

## Prerequisites

### All Platforms
- Node.js 18+
- Rust 1.77+ (`rustup update stable`)

### Linux (Ubuntu/Debian)
```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf libssl-dev
```

### Windows
- WebView2 (pre-installed on Windows 10/11)
- Visual Studio Build Tools with C++ workload

### macOS
- Xcode Command Line Tools (`xcode-select --install`)

## Icons

Before building, generate app icons from a 1024x1024 source PNG:

```bash
cd ~/FitAllDesktop
npx @tauri-apps/cli icon path/to/fitall-icon-1024x1024.png
```

This generates all required sizes in `src-tauri/icons/`.

Alternatively, copy placeholder icons manually:
- `32x32.png` (32x32)
- `128x128.png` (128x128)
- `128x128@2x.png` (256x256)
- `icon.icns` (macOS)
- `icon.ico` (Windows)
- `icon.png` (512x512 or 1024x1024)
- `tray-icon.png` (32x32, monochrome recommended for macOS)

## Development

```bash
cd ~/FitAllDesktop
npm install
npm run tauri:dev
```

This opens the desktop app window pointing to `http://localhost:1420`.
For full development with the FitAll frontend hot-reload, start the frontend dev server separately:

```bash
# Terminal 1: FitAll frontend
cd ~/FITALL/fitall_app/frontend
npm run dev -- --port 1420

# Terminal 2: Tauri dev shell
cd ~/FitAllDesktop
npm run tauri:dev
```

## Building

```bash
# Current platform
npm run tauri:build

# Platform-specific
npm run tauri:build:windows
npm run tauri:build:macos
npm run tauri:build:linux
```

Output goes to `src-tauri/target/release/bundle/`.

## CI/CD (GitHub Actions)

Push a version tag to trigger cross-platform builds:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This creates a draft GitHub Release with installers for all 3 platforms.

### Required Secrets

| Secret | Purpose |
|--------|---------|
| `TAURI_SIGNING_PRIVATE_KEY` | Signs update bundles |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Key password |

Generate signing keys:
```bash
cargo tauri signer generate -w ~/.tauri/fitall.key
```

## Architecture

```
FitAllDesktop/
├── package.json              # npm scripts
├── src/
│   └── index.html            # Loading screen (shown before remote loads)
├── src-tauri/
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # App config (window, tray, bundle, updater)
│   ├── capabilities/
│   │   └── default.json      # Security permissions
│   ├── icons/                # App icons (all sizes)
│   └── src/
│       ├── main.rs           # Entry point
│       └── lib.rs            # App logic (tray, IPC commands, health checks)
└── .github/workflows/
    └── release.yml           # Cross-platform CI/CD
```

### How It Works

1. App launches -> shows local `index.html` (loading screen with FitAll branding)
2. Checks `fitall.auraalpha.cc/api/health`
3. If reachable -> navigates webview to `https://fitall.auraalpha.cc`
4. If not -> shows connection error with retry button
5. System tray stays active when window is closed
6. Tray menu provides quick access to Dashboard, Quick Log, Workouts, Nutrition
7. Auto-update checks on startup via configured endpoint

### System Tray Menu

| Item | Action |
|------|--------|
| Open FitAll | Show/focus main window |
| Dashboard | Navigate to dashboard |
| Quick Log | Navigate to food/activity log |
| Workouts | Navigate to workouts page |
| Nutrition | Navigate to nutrition page |
| Check Health | Run API health check, show notification |
| Quit | Exit application |
