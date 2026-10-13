# Ultimate Windows Desktop Settings

One place for the Windows settings you change most: startup apps, sound, Night Light, File Explorer and taskbar options, Bluetooth, browsers, installed apps, timers and alarms, and quick notes.

Every feature is a bead on a silver rod. Hover a bead to magnify it and see its name; click it and it widens into its board, where you manage that setting. The title badge sits in the middle of the rod.

In the Windows app the rod floats on your desktop with no window, background or title bar around it. Press anywhere on it and drag to move it. Boards open below the rod when it sits high on the screen and above it when it sits low. Click the title badge for two buttons: minimize to the tray, or shut the app down.

In the browser preview the rod stands upright on narrow or tall windows and boards open full screen.

Version 2 is a rewrite of v1.5.0 (Flutter) with the same features done for real: a per-app volume mixer, a screen-wide Night Light, startup toggles that agree with Task Manager, and no PowerShell.

## Tech stack

The tech stack is listed here, and in the app under **About & Credits**.

| Layer | Technology | What it does |
| --- | --- | --- |
| App shell | **Tauri 2** | Native Windows window, tray icon, secure bridge to the system layer |
| System layer | **Rust** (2024 edition) | Registry, Core Audio, displays, services and files through Microsoft's `windows` crate |
| Interface | **React 19** | The silver rod, beads and boards |
| Language | **TypeScript 6** | Type-safe interface code |

Tooling: Vite 8, Motion, Fluent UI icons, Vitest and `cargo test`, Docker.

## Run it with Docker

You need Docker Desktop only.

### Design preview (any OS)

```sh
docker compose up -d
```

Docker Desktop then shows the `ultimate-windows-desktop-settings` container running in the background. Click its `8080:8080` port link, or open <http://localhost:8080>. A container cannot change the settings of your PC, so the preview uses sample data.

Stop it with `docker compose down`.

### The real Windows app

```sh
docker compose --profile windows run --rm windows-app
```

This cross-compiles the app inside Docker and puts `ultimate-windows-desktop-settings.exe` in the `out` folder. Run that file on Windows 10 or 11. The first build downloads the Rust toolchain and Microsoft's C runtime headers, so it takes a while; later builds are cached.

## Develop

Requirements: Node.js 22.22.2 or newer and Rust 1.85 or newer. On Windows, `npm run tauri dev` runs the real app with hot reload.

| Command | What it does |
| --- | --- |
| `npm run dev` | Interface only, in a browser, with sample data |
| `npm run tauri dev` | The full Windows app with hot reload (Windows only) |
| `npm run tauri build` | Windows installer in `target/release/bundle/nsis` |
| `npm run lint` / `npm run typecheck` / `npm test` | Interface checks |
| `cargo test` | Rust core tests (run on any OS against an in-memory registry) |

## Project layout

```
crates/uds-core   Rust: every Windows feature, testable without Windows
src-tauri         Rust: the Tauri app, tray, background alarms and commands
src               TypeScript + React: the rod, beads and boards
  boards/         one board per bead
  lib/api.ts      the only place the interface talks to Windows
  lib/mock.ts     sample data for the browser preview and tests
```

## Your data

Settings, alarms and notes live in `%APPDATA%\UltimateWindowsDesktopSettings`. Notes from v1 are imported automatically on first launch and never overwritten.

## Credits

Developed by **EdMaster28** (Software Developer & Computer Instructor) · dipaaapi8@gmail.com

Built with **Claude**, Anthropic's AI, through Claude Code. EdMaster28 gave the ideas, design and instructions; Claude wrote the code.
