// Sample data for the browser preview (Docker, `npm run dev`) and for tests.
// Nothing here touches the real PC; it only imitates the Rust commands.

import type {
  Alarm,
  AudioState,
  Browser,
  Countdown,
  Fired,
  InstalledApp,
  NoteMeta,
  PowerInfo,
  Preferences,
  ShortcutInfo,
  StartupItem,
  TweakState,
} from './types';
import { SAMPLE_SETTINGS } from './mockSettings';

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

type Listener = (event: Fired) => void;
const listeners = new Set<Listener>();
export const mockEvents = {
  subscribe(listener: Listener) {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },
  emit(event: Fired) {
    listeners.forEach((l) => l(event));
  },
};

function createState() {
  const preferences: Preferences = {
    theme: 'system',
    alwaysOnTop: false,
    closeToTray: true,
    nightLightEnabled: false,
    nightLightStrength: 40,
    worldClocks: ['Europe/London', 'America/New_York', 'Asia/Tokyo'],
    uiScale: 1,
  };
  const HKCU_RUN = 'HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run';
  const HKLM_RUN = 'HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run';
  const HKLM_RUN32 = 'HKLM\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run';
  const USER_FOLDER = 'C:\\Users\\Ed\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup';
  const item = (
    id: string,
    name: string,
    command: string,
    source: StartupItem['source'],
    location: string,
    extra: Partial<StartupItem> = {},
  ): StartupItem => ({ id, name, command, source, location, enabled: true, editable: true, removable: true, needsAdmin: false, ...extra });
  const startup: StartupItem[] = [
    item('user-run:Discord', 'Discord', 'C:\\Users\\Ed\\AppData\\Local\\Discord\\Update.exe --processStart Discord.exe', 'userRun', HKCU_RUN),
    item('user-run:Spotify', 'Spotify', '"C:\\Users\\Ed\\AppData\\Roaming\\Spotify\\Spotify.exe" /minimized', 'userRun', HKCU_RUN, { enabled: false }),
    item('user-run:OneDrive', 'OneDrive', '"C:\\Program Files\\Microsoft OneDrive\\OneDrive.exe" /background', 'userRun', HKCU_RUN),
    item('machine-run:SecurityHealth', 'SecurityHealth', '%windir%\\system32\\SecurityHealthSystray.exe', 'machineRun', HKLM_RUN, { needsAdmin: true }),
    item('machine-run32:Steam', 'Steam', '"C:\\Program Files (x86)\\Steam\\steam.exe" -silent', 'machineRun32', HKLM_RUN32, { needsAdmin: true }),
    item('machine-run32:FxSound', 'FxSound', '"C:\\Program Files\\FxSound LLC\\FxSound\\FxSound.exe"', 'machineRun32', HKLM_RUN32, { needsAdmin: true, enabled: false }),
    item('user-folder:Sticky Notes.lnk', 'Sticky Notes', `${USER_FOLDER}\\Sticky Notes.lnk`, 'userFolder', USER_FOLDER),
    item('common-folder:ShareMouse.lnk', 'ShareMouse', 'C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\StartUp\\ShareMouse.lnk', 'commonFolder', 'C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\StartUp', { needsAdmin: true }),
    item('store-app:SpotifyAB.SpotifyMusic_zpdnekdrzrea0\\SpotifyStartupTask', 'Spotify Music', 'SpotifyAB.SpotifyMusic_zpdnekdrzrea0!SpotifyStartupTask', 'storeApp', 'Microsoft Store app', { enabled: false, removable: false }),
    item('store-app:MSTeams_8wekyb3d8bbwe\\TeamsTfwStartupTask', 'MS Teams', 'MSTeams_8wekyb3d8bbwe!TeamsTfwStartupTask', 'storeApp', 'Microsoft Store app', { removable: false }),
    item('task:\\Opera GX scheduled assistant Autoupdate', 'Opera GX scheduled assistant Autoupdate', '"C:\\Users\\Ed\\AppData\\Local\\Programs\\Opera GX\\launcher.exe" --scheduledautoupdate', 'scheduledTask', 'Task Scheduler \\Opera GX scheduled assistant Autoupdate', { removable: false }),
    item('task:\\GoogleUpdaterTaskUser', 'GoogleUpdaterTaskUser', '"C:\\Users\\Ed\\AppData\\Local\\Google\\GoogleUpdater\\updater.exe" --wake', 'scheduledTask', 'Task Scheduler \\GoogleUpdaterTaskUser', { removable: false }),
    item('user-run-once:Setup cleanup', 'Setup cleanup', 'C:\\Users\\Ed\\AppData\\Local\\Temp\\cleanup.exe /finish', 'userRunOnce', 'HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce', { editable: false }),
  ].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
  const audio: AudioState = {
    master: 0.62,
    muted: false,
    devices: [
      { id: 'speakers', name: 'Speakers (Realtek(R) Audio)', isDefault: true },
      { id: 'hdmi', name: 'LG HDR WFHD (NVIDIA High Definition Audio)', isDefault: false },
      { id: 'buds', name: 'Headphones (WH-1000XM4)', isDefault: false },
    ],
    sessions: [
      { pid: 0, name: 'System sounds', volume: 0.8, muted: false, isSystem: true },
      { pid: 4120, name: 'Chrome', volume: 0.7, muted: false, isSystem: false },
      { pid: 7788, name: 'Discord', volume: 0.55, muted: false, isSystem: false },
      { pid: 9012, name: 'Spotify', volume: 0.9, muted: true, isSystem: false },
    ],
  };
  const settings: TweakState[] = structuredClone(SAMPLE_SETTINGS);
  // The preview pretends dark mode is on, like the screenshots.
  for (const t of settings) {
    if (t.id === 'apps-dark' || t.id === 'system-dark') Object.assign(t, { enabled: true, selected: 1 });
  }
  const power: PowerInfo = {
    plans: [
      { id: '381b4222-f694-41f0-9685-ff5bb260df2e', name: 'Balanced', active: true },
      { id: '8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c', name: 'High performance', active: false },
      { id: 'a1841308-3541-4fab-bc81-f71556f20b4a', name: 'Power saver', active: false },
    ],
    battery: { hasBattery: true, percent: 76, pluggedIn: true, charging: true, saverOn: false },
  };
  const browsers: Browser[] = [
    { id: 'Google Chrome', name: 'Google Chrome', exe: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe', kind: 'chrome', isDefault: true, supportsPrivate: true },
    { id: 'Brave', name: 'Brave', exe: 'C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe', kind: 'brave', isDefault: false, supportsPrivate: true },
    { id: 'Firefox-308046B0AF4A39CB', name: 'Firefox', exe: 'C:\\Program Files\\Mozilla Firefox\\firefox.exe', kind: 'firefox', isDefault: false, supportsPrivate: true },
    { id: 'Microsoft Edge', name: 'Microsoft Edge', exe: 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe', kind: 'edge', isDefault: false, supportsPrivate: true },
    { id: 'OperaStable', name: 'Opera Stable', exe: 'C:\\Users\\Ed\\AppData\\Local\\Programs\\Opera\\opera.exe', kind: 'opera', isDefault: false, supportsPrivate: true },
  ];
  const now = Date.now();
  const notes = new Map<string, { content: string; pinned: boolean; modifiedMs: number }>([
    ['General Notes', { content: 'Welcome to UDS Scrapbook Notes!\nWrite your notes here. They are saved on this PC only.', pinned: true, modifiedMs: now - 3600_000 }],
    ['Project ideas', { content: '- Silver rod layout\n- Bead magnification like the Dock\n- Docker preview', pinned: false, modifiedMs: now - 600_000 }],
    ['Shopping', { content: 'HDMI cable, mouse pad', pinned: false, modifiedMs: now - 86_400_000 }],
  ]);
  const apps: InstalledApp[] = [
    ['Aseprite', 'Igara Studio', 'Design & Creative'],
    ['Blender 4.5', 'Blender Foundation', 'Design & Creative'],
    ['Docker Desktop', 'Docker Inc.', 'Development'],
    ['Git', 'The Git Development Community', 'Development'],
    ['Microsoft Visual Studio Code (User)', 'Microsoft Corporation', 'Development'],
    ['Node.js', 'Node.js Foundation', 'Development'],
    ['Steam', 'Valve Corporation', 'Gaming'],
    ['Riot Client', 'Riot Games, Inc', 'Gaming'],
    ['DS4Windows', 'Ryochan7', 'Gaming'],
    ['Zoom Workplace', 'Zoom Communications, Inc.', 'Office & General Use'],
    ['VLC media player', 'VideoLAN', 'Office & General Use'],
    ['Microsoft 365 Apps for enterprise', 'Microsoft Corporation', 'Office & General Use'],
  ].map(([name, publisher, category], i) => ({
    id: `HKLM:app-${i}`,
    name,
    version: `${1 + (i % 5)}.${i}.0`,
    publisher,
    installLocation: `C:\\Program Files\\${name.split(' ')[0]}`,
    exe: i % 4 === 3 ? null : `C:\\Program Files\\${name.split(' ')[0]}\\${name.split(' ')[0].toLowerCase()}.exe`,
    category: category as InstalledApp['category'],
  }));
  const shortcuts: ShortcutInfo[] = [
    ['display', 'Display', 'Resolution, scale, brightness and multiple monitors.', 'System'],
    ['power', 'Power & battery', 'Power mode, battery saver and battery usage.', 'System'],
    ['wifi', 'Wi-Fi', 'Networks, known networks and hardware properties.', 'Network & internet'],
    ['sign-in', 'Sign-in options', 'PIN, Windows Hello, password and dynamic lock.', 'Accounts'],
    ['magnifier', 'Magnifier', 'Zoom in on parts of the screen.', 'Accessibility'],
    ['printers', 'Printers & scanners', 'Add printers and set the default one.', 'Bluetooth & devices'],
    ['night-light', 'Night light', "Windows' own night light schedule.", 'Windows Settings'],
    ['sound', 'Sound', 'Output and input devices and system sounds.', 'Windows Settings'],
    ['app-volume', 'App volume and device preferences', 'Pick an output device per app.', 'Windows Settings'],
    ['bluetooth', 'Bluetooth & devices', 'Pair and manage Bluetooth devices.', 'Windows Settings'],
    ['network', 'Network & internet', 'Wi-Fi, Ethernet, VPN and DNS.', 'Windows Settings'],
    ['personalization', 'Personalization', 'Background, colors, themes and lock screen.', 'Windows Settings'],
    ['taskbar', 'Taskbar', 'Taskbar items, behaviors and alignment.', 'Windows Settings'],
    ['notifications', 'Notifications', 'Notification and Do Not Disturb settings.', 'Windows Settings'],
    ['apps', 'Installed apps', 'Modify or uninstall programs.', 'Windows Settings'],
    ['default-apps', 'Default apps', 'Choose the default browser and other apps.', 'Windows Settings'],
    ['startup-apps', 'Startup apps', "Windows' own startup apps page.", 'Windows Settings'],
    ['date-time', 'Date & time', 'Time zone, clock sync and formats.', 'Windows Settings'],
    ['windows-update', 'Windows Update', 'Updates, update history and restarts.', 'Windows Settings'],
    ['privacy', 'Privacy & security', 'App permissions, Windows Security and firewall.', 'Windows Settings'],
    ['about', 'About this PC', 'Processor, memory and Windows version.', 'Windows Settings'],
    ['troubleshoot', 'Troubleshoot', 'Windows troubleshooters, including audio.', 'Windows Settings'],
    ['control-panel', 'Control Panel', 'The classic Control Panel.', 'System Tools'],
    ['task-manager', 'Task Manager', 'Running apps, performance and startup.', 'System Tools'],
    ['device-manager', 'Device Manager', 'Hardware and drivers.', 'System Tools'],
    ['services', 'Services', 'Start, stop and configure Windows services.', 'System Tools'],
    ['system-info', 'System Information', 'Detailed hardware and software report.', 'System Tools'],
    ['disk-cleanup', 'Disk Cleanup', 'Free up space on a drive.', 'System Tools'],
    ['command-prompt', 'Command Prompt', 'Open a command prompt window.', 'System Tools'],
    ['sound-devices', 'Sound devices', 'Classic playback and recording devices panel.', 'System Tools'],
    ['volume-mixer', 'Volume Mixer', 'Classic per-app volume mixer.', 'System Tools'],
    ['bluetooth-pair', 'Add a Bluetooth device', 'Windows pairing wizard.', 'System Tools'],
    ['bluetooth-files', 'Send or receive files', 'Bluetooth file transfer.', 'System Tools'],
    ['recycle-bin', 'Recycle Bin', 'Open the Recycle Bin folder.', 'Folders & Apps'],
    ['temp-folder', 'Temp folder', 'Open your %TEMP% folder.', 'Folders & Apps'],
    ['startup-folder', 'Startup folder', 'Shortcuts here start when you sign in.', 'Folders & Apps'],
    ['sticky-notes', 'Sticky Notes', 'Microsoft Sticky Notes.', 'Folders & Apps'],
    ['notepad', 'Notepad', 'Windows Notepad.', 'Folders & Apps'],
    ['calculator', 'Calculator', 'Windows Calculator.', 'Folders & Apps'],
  ].map(([id, title, description, group]) => ({ id, title, description, group }));

  return {
    preferences,
    launchAtStartup: false,
    startup,
    audio,
    settings,
    power,
    wallpaper: 'C:\\Windows\\Web\\Wallpaper\\Windows\\img0.jpg' as string | null,
    browsers,
    notes,
    apps,
    shortcuts,
    alarms: [
      { id: 'alarm-1', label: 'Morning class', hour: 6, minute: 30, days: [1, 2, 3, 4, 5], enabled: true, tone: 'default' },
      { id: 'alarm-2', label: 'Stretch break', hour: 15, minute: 0, days: [], enabled: false, tone: 'information' },
    ] as Alarm[],
    countdown: null as Countdown | null,
    countdownTimer: undefined as ReturnType<typeof setTimeout> | undefined,
    temp: { items: 214, bytes: 1_382_400_000 },
    recycleBin: { items: 37, bytes: 512_000_000 },
    bluetoothService: 'running' as 'running' | 'stopped',
  };
}

let state = createState();

/** Resets the sample data (used by tests). */
export function resetMock() {
  if (state.countdownTimer) clearTimeout(state.countdownTimer);
  state = createState();
}

function noteList(): NoteMeta[] {
  return [...state.notes.entries()]
    .map(([title, n]) => ({ title, pinned: n.pinned, modifiedMs: n.modifiedMs, preview: n.content.replace(/\s+/g, ' ').slice(0, 90) }))
    .sort((a, b) => Number(b.pinned) - Number(a.pinned) || a.title.toLowerCase().localeCompare(b.title.toLowerCase()));
}

function validateTitle(title: string): string {
  const t = title.trim();
  if (!t) throw new Error('Note title cannot be empty.');
  if (/[\\/:*?"<>|]/.test(t)) throw new Error('Note title cannot contain \\ / : * ? " < > |');
  return t;
}

const clone = <T>(value: T): T => structuredClone(value);

export async function mockCall<T>(command: string, args: Record<string, unknown>): Promise<T> {
  await delay(import.meta.env.MODE === 'test' ? 0 : 140);
  return clone(handle(command, args)) as T;
}

function handle(command: string, args: Record<string, unknown>): unknown {
  const s = state;
  switch (command) {
    case 'app_info':
      return { version: '2.1.0', dataDir: 'C:\\Users\\Ed\\AppData\\Roaming\\UltimateWindowsDesktopSettings', platform: 'preview' };
    case 'get_preferences':
      return s.preferences;
    case 'set_preferences':
      s.preferences = args.preferences as Preferences;
      return s.preferences;
    case 'get_launch_at_startup':
      return s.launchAtStartup;
    case 'set_launch_at_startup':
      s.launchAtStartup = Boolean(args.enabled);
      return s.launchAtStartup;
    case 'overview':
      return {
        startupTotal: s.startup.length,
        startupEnabled: s.startup.filter((i) => i.enabled).length,
        bluetoothDevices: 3,
        temp: s.temp,
        recycleBin: s.recycleBin,
      };
    case 'refresh_desktop':
      return null;
    case 'temp_summary':
      return s.temp;
    case 'clean_temp': {
      const result = { removed: 180, skipped: 34, freedBytes: 1_210_000_000 };
      s.temp = { items: 34, bytes: s.temp.bytes - result.freedBytes };
      return result;
    }
    case 'empty_recycle_bin':
      s.recycleBin = { items: 0, bytes: 0 };
      return s.recycleBin;
    case 'list_startup':
      return s.startup;
    case 'set_startup_enabled': {
      const item = s.startup.find((i) => i.id === args.id);
      if (!item) throw new Error('Not found');
      if (!item.editable) throw new Error('This entry runs only once. Remove it instead if you don\'t want it to run.');
      item.enabled = Boolean(args.enabled);
      return s.startup;
    }
    case 'add_startup': {
      const name = String(args.name).trim();
      const cmd = String(args.command).trim();
      if (!name || !cmd) throw new Error('Name and program path cannot be empty.');
      if (s.startup.some((i) => i.source === 'userRun' && i.name.toLowerCase() === name.toLowerCase())) throw new Error(`A startup entry named "${name}" already exists.`);
      s.startup.push({ id: `user-run:${name}`, name, command: cmd, source: 'userRun', enabled: true, editable: true, removable: true, needsAdmin: false, location: 'HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' });
      s.startup.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
      return s.startup;
    }
    case 'remove_startup': {
      const item = s.startup.find((i) => i.id === args.id);
      if (!item) throw new Error('Not found');
      if (!item.removable) throw new Error('This entry can only be turned off here.');
      s.startup = s.startup.filter((i) => i.id !== args.id);
      return s.startup;
    }
    case 'audio_state':
      return s.audio;
    case 'set_master_volume':
      s.audio.master = Number(args.levelValue);
      return s.audio;
    case 'set_master_mute':
      s.audio.muted = Boolean(args.muted);
      return s.audio;
    case 'set_app_volume':
    case 'set_app_mute': {
      const session = s.audio.sessions.find((x) => x.pid === args.pid);
      if (!session) throw new Error('That app is no longer playing sound.');
      if (command === 'set_app_volume') session.volume = Number(args.levelValue);
      else session.muted = Boolean(args.muted);
      return s.audio;
    }
    case 'list_displays':
      return [
        { name: 'Built-in Display', device: '\\\\.\\DISPLAY1', width: 1920, height: 1080, refreshHz: 144, scalePercent: 125, primary: true },
        { name: 'LG HDR WFHD', device: '\\\\.\\DISPLAY2', width: 2560, height: 1080, refreshHz: 75, scalePercent: 100, primary: false },
      ];
    case 'list_settings':
      return s.settings.filter((t) => t.page === args.page);
    case 'set_setting': {
      const t = s.settings.find((x) => x.id === args.id);
      if (!t) throw new Error(`Not found: setting ${String(args.id)}`);
      const value = Number(args.value);
      if (t.range) {
        if (!(value >= t.range.min && value <= t.range.max)) throw new Error(`${t.title} can't be ${value}`);
        t.value = value;
      } else {
        if (!Number.isInteger(value) || value < 0 || value >= t.options.length) throw new Error(`Invalid input: option ${value} of ${t.id}`);
        t.selected = value;
        t.enabled = t.toggle && value === 1;
      }
      return s.settings.filter((x) => x.page === t.page);
    }
    case 'restart_explorer':
      return null;
    case 'power_info':
      return s.power;
    case 'set_power_plan':
      if (!s.power.plans.some((p) => p.id === args.id)) throw new Error('Not found: power plan');
      s.power.plans.forEach((p) => (p.active = p.id === args.id));
      return s.power;
    case 'get_wallpaper':
      return s.wallpaper;
    case 'set_wallpaper':
      s.wallpaper = String(args.path);
      return s.wallpaper;
    case 'bluetooth_state':
      return {
        service: s.bluetoothService,
        devices: [
          { address: '00:1A:7D:DA:71:13', name: 'WH-1000XM4' },
          { address: 'A0:B1:C2:D3:E4:F5', name: 'Logitech MX Keys' },
          { address: '7C:2A:31:0F:9B:44', name: 'DualShock 4' },
        ],
      };
    case 'control_bluetooth':
      s.bluetoothService = args.start ? 'running' : 'stopped';
      return null;
    case 'list_browsers':
      return s.browsers;
    case 'launch_browser':
      return null;
    case 'list_notes':
      return noteList();
    case 'read_note': {
      const n = s.notes.get(String(args.title));
      if (!n) throw new Error('Note not found');
      return n.content;
    }
    case 'save_note': {
      const n = s.notes.get(String(args.title));
      if (!n) throw new Error('Note not found');
      n.content = String(args.content);
      n.modifiedMs = Date.now();
      return null;
    }
    case 'create_note': {
      const title = validateTitle(String(args.title));
      if ([...s.notes.keys()].some((k) => k.toLowerCase() === title.toLowerCase())) throw new Error(`A note named "${title}" already exists.`);
      s.notes.set(title, { content: '', pinned: false, modifiedMs: Date.now() });
      return title;
    }
    case 'rename_note': {
      const from = String(args.title);
      const to = validateTitle(String(args.newTitle));
      const n = s.notes.get(from);
      if (!n) throw new Error('Note not found');
      if (from.toLowerCase() !== to.toLowerCase() && s.notes.has(to)) throw new Error(`A note named "${to}" already exists.`);
      s.notes.delete(from);
      s.notes.set(to, n);
      return to;
    }
    case 'delete_note': {
      const n = s.notes.get(String(args.title));
      if (!n) throw new Error('Note not found');
      if (n.pinned) throw new Error('Unpin this note before deleting it.');
      if (s.notes.size <= 1) throw new Error('You must keep at least one note.');
      s.notes.delete(String(args.title));
      return null;
    }
    case 'pin_note': {
      const n = s.notes.get(String(args.title));
      if (!n) throw new Error('Note not found');
      n.pinned = Boolean(args.pinned);
      return null;
    }
    case 'open_notes_folder':
    case 'open_shortcut':
    case 'launch_app':
    case 'open_app_folder':
      return null;
    case 'list_apps':
      return s.apps;
    case 'close_app':
      return 1;
    case 'list_shortcuts':
      return s.shortcuts;
    case 'list_alarms':
      return s.alarms;
    case 'save_alarm': {
      const alarm = args.alarm as Alarm;
      const i = s.alarms.findIndex((a) => a.id === alarm.id);
      if (i >= 0) s.alarms[i] = alarm;
      else s.alarms.push(alarm);
      s.alarms.sort((a, b) => a.hour * 60 + a.minute - (b.hour * 60 + b.minute));
      return s.alarms;
    }
    case 'delete_alarm':
      s.alarms = s.alarms.filter((a) => a.id !== args.id);
      return s.alarms;
    case 'test_tone':
      return null;
    case 'start_countdown': {
      const seconds = Number(args.seconds);
      if (!(seconds > 0 && seconds <= 86400)) throw new Error('Timer must be between 1 second and 24 hours.');
      if (s.countdownTimer) clearTimeout(s.countdownTimer);
      const label = String(args.label);
      s.countdown = { label, endsAtMs: Date.now() + seconds * 1000 };
      s.countdownTimer = setTimeout(() => {
        state.countdown = null;
        mockEvents.emit({ kind: 'countdown', label });
      }, seconds * 1000);
      return s.countdown;
    }
    case 'cancel_countdown':
      if (s.countdownTimer) clearTimeout(s.countdownTimer);
      s.countdown = null;
      return null;
    case 'set_window_mode':
    case 'window_placement':
      return 'below';
    case 'start_dragging':
    case 'hide_to_tray':
    case 'quit_app':
      return undefined;
    case 'countdown_state':
      return s.countdown;
    default:
      throw new Error(`Unknown command: ${command}`);
  }
}
