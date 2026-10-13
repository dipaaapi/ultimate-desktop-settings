# Ultimate Windows Desktop Settings

Windows settings utility. Tauri 2 + Rust (2024) + React 19 + TypeScript 6. The repo's language bar must show only Rust and TypeScript: no PowerShell or other languages in the app, and new non-code file types get a `linguist-detectable=false` line in `.gitattributes`.

## Layout
- `crates/uds-core`: all Windows logic. Platform-neutral code is tested against `MemoryRegistry` and temp dirs; real Windows calls live in `src/win/` and are reached through `system.rs`, which returns `Error::Unsupported` off Windows.
- `src-tauri`: Tauri shell. `commands.rs` holds every command the UI can call (async, so the window never blocks).
- `src`: the interface. `lib/api.ts` is the only bridge; outside Tauri it answers from `lib/mock.ts`. Keep the two in sync when adding a command.
- Beads are defined in `src/beads.ts`; category colors match the logo (`#5EEAD4`, `#60A5FA`, `#C084FC`, `#FCD34D`). The logo files are owned separately; don't redesign them.

## Checks before pushing
- `npm run lint && npm run typecheck && npm test && npm run build`
- `cargo fmt --all --check && cargo clippy -p uds-core --all-targets -- -D warnings && cargo test`
- From Linux, the Windows target can be checked with `cargo clippy --workspace --target x86_64-pc-windows-gnu` (needs mingw-w64) and tested with `cargo test -p uds-core --target x86_64-pc-windows-gnu` (needs wine).

## Rules
- Never shell out to PowerShell; use the `windows` crate.
- Destructive actions (cleaning temp, emptying the Recycle Bin, removing entries) need a confirm step in the UI.
- Close apps gracefully (WM_CLOSE), never kill them.
- The desktop app is a floating rod: transparent, no title bar, no background (`.desktop` CSS). The window grows only while a board is open, towards the roomier side (`uds-core/src/window_layout.rs`). Minimize-to-tray and shut down live on the title badge.
- The browser preview must still hold at any window size: horizontal rod when wide, vertical when narrow or tall (`lib/layout.ts`).
