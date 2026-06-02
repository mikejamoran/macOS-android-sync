# Android Drop Sync

A lightweight, fast Mac desktop application designed to easily transfer files (like ROMs) and install APKs to Android handheld consoles via USB.

> **Note:** This project was developed with the assistance of **Gemini CLI**. This may be a no-go for some users, and that's absolutely fine. Generative AI is bad for the vast majority of use cases and should never be used to generate creative pursuits such as artwork, music or writing. This app has utilised it in development here for something I've not been happy with in existing apps. However, this is still an example of using AI in coding so please bear this in mind before you use it -- I am uploading this in case it is of interest, but I appreciate fully that AI is a red line for many people. 

## Key Features
- **Junk Filtering:** Automatically strips macOS hidden files (`.DS_Store`, `._*`, `.Trashes`) during transfer.
- **Smart Routing:** APKs are automatically installed; ROMs and other files go to your chosen destination.
- **Remote Browser:** Navigate and select folders directly on your Android device.
- **Zero Config:** Bundled ADB for Apple Silicon Mac.

## Getting Started: Enabling USB Debugging
To use this app, your Android device must have USB Debugging enabled:

1.  **Open Settings:** On your Android device, go to **Settings**.
2.  **Find "About Device":** Scroll to the bottom and tap **About Device** (or About Tablet/Phone).
3.  **Unlock Developer Options:** Find **Build Number** and tap it **7 times** until it says "You are now a developer!"
4.  **Enable Debugging:** Go back to Settings > System > **Developer Options** and toggle **USB Debugging** to **ON**.
5.  **Authorize Mac:** Connect the device to your Mac via USB and tap **Allow** (and "Always allow") when the popup appears on the Android screen.

## Development
- `npm install` to install dependencies.
- `npm run tauri dev` to run in development mode.
- `npm run tauri build` to build the standalone `.app`.

## Project Details
For more technical information on the architecture and build process, see [PROJECT_DETAILS.md](./PROJECT_DETAILS.md).
