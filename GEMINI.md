# Android Drop Sync - Project Context

## Architecture
This is a desktop application built with **Tauri** (Rust backend) and **Svelte** (TypeScript frontend). It is designed to facilitate file transfers to Android handheld game consoles via ADB.

## Key Features
- **ADB Sidecar:** Uses a bundled macOS `adb` binary (`src-tauri/binaries/adb-aarch64-apple-darwin`).
- **Junk Filtering:** Automatically strips macOS hidden files (`.DS_Store`, `._*`, `.Trashes`) during directory push.
- **Smart Routing:** APKs are automatically installed; all other files go to a user-specified path (default `/sdcard/Download/`).
- **Remote Browser:** Custom ADB-based directory browser to select destination folders on the device.

## Commands
- `npm run tauri dev`: Start the app in development mode.
- `npm run tauri build`: Build the standalone macOS `.app` bundle.

## Tech Stack
- **Backend:** Rust (Tauri v2)
- **Frontend:** Svelte 5 + TypeScript
- **Styling:** Vanilla CSS (Dark Mode optimized)
- **Plugins:** 
  - `tauri-plugin-shell` (for ADB execution)
  - `tauri-plugin-dialog` (for local file selection)
  - `tauri-plugin-opener` (default tauri plugin)

## Development Notes
- The ADB binary is targeted for `aarch64-apple-darwin` (Apple Silicon).
- Remote directory listing uses `ls -F` via ADB shell to distinguish folders from files.
