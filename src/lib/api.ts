// The only way the UI talks to Windows. Inside the desktop app every call goes
// to a Rust command; in a normal browser (the Docker preview, `npm run dev`
// and tests) the same calls are answered by the sample data in ./mock.ts.

import type {
  Alarm,
  AppInfo,
  AudioState,
  BluetoothState,
  Browser,
  CleanResult,
  Countdown,
  DisplayInfo,
  Fired,
  InstalledApp,
  NoteMeta,
  Overview,
  Placement,
  Preferences,
  RecycleBinSummary,
  ShortcutInfo,
  StartupItem,
  TempSummary,
  Tone,
  TweakState,
} from './types';

export const isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

type Args = Record<string, unknown>;

async function call<T>(command: string, args: Args = {}): Promise<T> {
  if (isDesktop) {
    const { invoke } = await import('@tauri-apps/api/core');
    try {
      return await invoke<T>(command, args);
    } catch (error) {
      throw new Error(typeof error === 'string' ? error : String(error), { cause: error });
    }
  }
  const { mockCall } = await import('./mock');
  return mockCall<T>(command, args);
}

export const api = {
  appInfo: () => call<AppInfo>('app_info'),
  getPreferences: () => call<Preferences>('get_preferences'),
  setPreferences: (preferences: Preferences) => call<Preferences>('set_preferences', { preferences }),
  getLaunchAtStartup: () => call<boolean>('get_launch_at_startup'),
  setLaunchAtStartup: (enabled: boolean) => call<boolean>('set_launch_at_startup', { enabled }),

  overview: () => call<Overview>('overview'),
  refreshDesktop: () => call<void>('refresh_desktop'),
  tempSummary: () => call<TempSummary>('temp_summary'),
  cleanTemp: () => call<CleanResult>('clean_temp'),
  emptyRecycleBin: () => call<RecycleBinSummary>('empty_recycle_bin'),

  listStartup: () => call<StartupItem[]>('list_startup'),
  setStartupEnabled: (id: string, enabled: boolean) => call<StartupItem[]>('set_startup_enabled', { id, enabled }),
  addStartup: (name: string, command: string) => call<StartupItem[]>('add_startup', { name, command }),
  removeStartup: (id: string) => call<StartupItem[]>('remove_startup', { id }),

  audioState: () => call<AudioState>('audio_state'),
  setMasterVolume: (level: number) => call<AudioState>('set_master_volume', { levelValue: level }),
  setMasterMute: (muted: boolean) => call<AudioState>('set_master_mute', { muted }),
  setAppVolume: (pid: number, level: number) => call<AudioState>('set_app_volume', { pid, levelValue: level }),
  setAppMute: (pid: number, muted: boolean) => call<AudioState>('set_app_mute', { pid, muted }),

  listDisplays: () => call<DisplayInfo[]>('list_displays'),
  listTweaks: () => call<TweakState[]>('list_tweaks'),
  setTweak: (id: string, enabled: boolean) => call<TweakState[]>('set_tweak', { id, enabled }),

  bluetoothState: () => call<BluetoothState>('bluetooth_state'),
  controlBluetooth: (start: boolean) => call<void>('control_bluetooth', { start }),

  listBrowsers: () => call<Browser[]>('list_browsers'),
  launchBrowser: (id: string, isPrivate: boolean) => call<void>('launch_browser', { id, private: isPrivate }),

  listNotes: () => call<NoteMeta[]>('list_notes'),
  readNote: (title: string) => call<string>('read_note', { title }),
  saveNote: (title: string, content: string) => call<void>('save_note', { title, content }),
  createNote: (title: string) => call<string>('create_note', { title }),
  renameNote: (title: string, newTitle: string) => call<string>('rename_note', { title, newTitle }),
  deleteNote: (title: string) => call<void>('delete_note', { title }),
  pinNote: (title: string, pinned: boolean) => call<void>('pin_note', { title, pinned }),
  openNotesFolder: () => call<void>('open_notes_folder'),

  listApps: () => call<InstalledApp[]>('list_apps'),
  launchApp: (id: string) => call<void>('launch_app', { id }),
  openAppFolder: (id: string) => call<void>('open_app_folder', { id }),
  closeApp: (id: string) => call<number>('close_app', { id }),

  listShortcuts: () => call<ShortcutInfo[]>('list_shortcuts'),
  openShortcut: (id: string) => call<void>('open_shortcut', { id }),

  listAlarms: () => call<Alarm[]>('list_alarms'),
  saveAlarm: (alarm: Alarm) => call<Alarm[]>('save_alarm', { alarm }),
  deleteAlarm: (id: string) => call<Alarm[]>('delete_alarm', { id }),
  testTone: (tone: Tone) => call<void>('test_tone', { tone }),
  startCountdown: (seconds: number, label: string) => call<Countdown>('start_countdown', { seconds, label }),
  cancelCountdown: () => call<void>('cancel_countdown'),
  countdownState: () => call<Countdown | null>('countdown_state'),

  // The floating window (desktop app only; the preview answers with defaults).
  setWindowMode: (mode: 'rod' | 'board') => call<Placement>('set_window_mode', { mode }),
  windowPlacement: () => call<Placement>('window_placement'),
  startDragging: () => call<void>('start_dragging'),
  hideToTray: () => call<void>('hide_to_tray'),
  quitApp: () => call<void>('quit_app'),
};

/** Lets the user pick a program for a new startup entry. */
export async function pickExecutable(): Promise<string | null> {
  if (!isDesktop) return 'C:\\Program Files\\Example\\example.exe';
  const { open } = await import('@tauri-apps/plugin-dialog');
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Programs', extensions: ['exe', 'bat', 'cmd', 'lnk'] }],
  });
  return typeof picked === 'string' ? picked : null;
}

/** Subscribes to alarms and timers ringing in the background. */
export async function onTimerFired(handler: (event: Fired) => void): Promise<() => void> {
  if (isDesktop) {
    const { listen } = await import('@tauri-apps/api/event');
    return listen<Fired>('timer-fired', (e) => handler(e.payload));
  }
  const { mockEvents } = await import('./mock');
  return mockEvents.subscribe(handler);
}
