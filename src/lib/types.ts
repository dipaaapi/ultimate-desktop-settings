// Mirrors the Rust types in crates/uds-core (serialized as camelCase).

export type ThemePreference = 'system' | 'light' | 'dark';

export interface Preferences {
  theme: ThemePreference;
  alwaysOnTop: boolean;
  closeToTray: boolean;
  nightLightEnabled: boolean;
  nightLightStrength: number;
  worldClocks: string[];
}

export interface AppInfo {
  version: string;
  dataDir: string;
  platform: string;
}

export interface TempSummary {
  items: number;
  bytes: number;
}

export interface CleanResult {
  removed: number;
  skipped: number;
  freedBytes: number;
}

export interface RecycleBinSummary {
  items: number;
  bytes: number;
}

export interface Overview {
  startupTotal: number;
  startupEnabled: number;
  bluetoothDevices: number;
  temp: TempSummary;
  recycleBin: RecycleBinSummary;
}

export type StartupSource = 'userRun' | 'machineRun' | 'userFolder';

export interface StartupItem {
  id: string;
  name: string;
  command: string;
  source: StartupSource;
  enabled: boolean;
  editable: boolean;
}

export interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface AudioSession {
  pid: number;
  name: string;
  volume: number;
  muted: boolean;
  isSystem: boolean;
}

export interface AudioState {
  master: number;
  muted: boolean;
  devices: AudioDevice[];
  sessions: AudioSession[];
}

export interface DisplayInfo {
  name: string;
  device: string;
  width: number;
  height: number;
  refreshHz: number;
  scalePercent: number;
  primary: boolean;
}

export interface TweakState {
  id: string;
  title: string;
  description: string;
  group: string;
  enabled: boolean;
}

export type ServiceState = 'running' | 'stopped' | 'starting' | 'stopping' | 'unknown';

export interface PairedDevice {
  address: string;
  name: string;
}

export interface BluetoothState {
  service: ServiceState;
  devices: PairedDevice[];
}

export type BrowserKind = 'chrome' | 'edge' | 'firefox' | 'brave' | 'opera' | 'vivaldi' | 'other';

export interface Browser {
  id: string;
  name: string;
  exe: string;
  kind: BrowserKind;
  isDefault: boolean;
  supportsPrivate: boolean;
}

export interface NoteMeta {
  title: string;
  pinned: boolean;
  modifiedMs: number;
  preview: string;
}

export type AppCategory = 'Development' | 'Design & Creative' | 'Gaming' | 'Office & General Use';

export interface InstalledApp {
  id: string;
  name: string;
  version: string | null;
  publisher: string | null;
  installLocation: string | null;
  exe: string | null;
  category: AppCategory;
}

export interface ShortcutInfo {
  id: string;
  title: string;
  description: string;
  group: string;
}

export type Tone = 'default' | 'information' | 'warning' | 'critical';

export interface Alarm {
  id: string;
  label: string;
  hour: number;
  minute: number;
  /** 0 = Sunday ... 6 = Saturday. Empty = rings once. */
  days: number[];
  enabled: boolean;
  tone: Tone;
}

export interface Countdown {
  label: string;
  endsAtMs: number;
}

export type Fired = { kind: 'alarm'; id: string; label: string; tone: Tone } | { kind: 'countdown'; label: string };
