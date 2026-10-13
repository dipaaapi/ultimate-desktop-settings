// The tech stack shown in About & Credits. Keep in sync with package.json and Cargo.toml.

export interface StackEntry {
  layer: string;
  name: string;
  version: string;
  role: string;
}

export const TECH_STACK: StackEntry[] = [
  { layer: 'App shell', name: 'Tauri', version: '2', role: 'Native Windows window, tray icon and secure bridge to the system layer' },
  { layer: 'System layer', name: 'Rust', version: '2024 edition', role: 'Registry, Core Audio, displays, services and files through the official windows crate' },
  { layer: 'Interface', name: 'React', version: '19', role: 'The silver rod, beads and boards' },
  { layer: 'Language', name: 'TypeScript', version: '6', role: 'Type-safe interface code' },
];

export const TOOLING: string[] = ['Vite 8 (build)', 'Motion (animation)', 'Fluent UI icons', 'Vitest & cargo test (tests)', 'Docker (preview & Windows build)'];
