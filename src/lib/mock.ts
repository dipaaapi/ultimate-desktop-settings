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
  Preferences,
  ShortcutInfo,
  StartupItem,
  TweakState,
} from './types';

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
  };
  const startup: StartupItem[] = [
    { id: 'user-run:Discord', name: 'Discord', command: 'C:\\Users\\Ed\\AppData\\Local\\Discord\\Update.exe --processStart Discord.exe', source: 'userRun', enabled: true, editable: true },
    { id: 'user-run:Spotify', name: 'Spotify', command: '"C:\\Users\\Ed\\AppData\\Roaming\\Spotify\\Spotify.exe" /minimized', source: 'userRun', enabled: false, editable: true },
    { id: 'user-run:OneDrive', name: 'OneDrive', command: '"C:\\Program Files\\Microsoft OneDrive\\OneDrive.exe" /background', source: 'userRun', enabled: true, editable: true },
    { id: 'machine-run:SecurityHealth', name: 'SecurityHealth', command: '%windir%\\system32\\SecurityHealthSystray.exe', source: 'machineRun', enabled: true, editable: false },
    { id: 'user-folder:Sticky Notes.lnk', name: 'Sticky Notes', command: 'C:\\Users\\Ed\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\Sticky Notes.lnk', source: 'userFolder', enabled: true, editable: true },
  ];
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
  const tweaks: TweakState[] = [
    { id: 'apps-dark', title: 'Dark mode for apps', description: 'Apps that follow the Windows theme use dark colors.', group: 'Theme', enabled: true },
    { id: 'system-dark', title: 'Dark mode for Windows', description: 'Taskbar, Start menu and notifications use dark colors.', group: 'Theme', enabled: true },
    { id: 'transparency', title: 'Transparency effects', description: 'Windows and surfaces appear slightly see-through.', group: 'Theme', enabled: true },
    { id: 'file-extensions', title: 'Show file extensions', description: 'File Explorer shows endings like .exe, .pdf and .txt.', group: 'File Explorer', enabled: false },
    { id: 'hidden-files', title: 'Show hidden files', description: 'File Explorer shows files and folders marked hidden.', group: 'File Explorer', enabled: false },
    { id: 'open-this-pc', title: 'Open File Explorer to This PC', description: 'New Explorer windows start at This PC instead of Home.', group: 'File Explorer', enabled: false },
    { id: 'compact-view', title: 'Compact view', description: 'Tighter spacing between files in File Explorer.', group: 'File Explorer', enabled: false },
    { id: 'taskbar-left', title: 'Align taskbar to the left', description: 'Start and pinned apps sit on the left like Windows 10.', group: 'Taskbar', enabled: false },
    { id: 'task-view-button', title: 'Task View button', description: 'Shows the Task View button on the taskbar.', group: 'Taskbar', enabled: true },
    { id: 'clock-seconds', title: 'Show seconds in the clock', description: 'The taskbar clock shows seconds (Windows 11).', group: 'Taskbar', enabled: false },
    { id: 'end-task', title: 'End task on right-click', description: 'Right-clicking a taskbar app offers End task (Windows 11).', group: 'Taskbar', enabled: false },
  ];
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
    ['display', 'Display', 'Resolution, scale, brightness and multiple monitors.', 'Windows Settings'],
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
    tweaks,
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
      return { version: '2.0.0', dataDir: 'C:\\Users\\Ed\\AppData\\Roaming\\UltimateWindowsDesktopSettings', platform: 'preview' };
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
      if (!item.editable) throw new Error('This entry is installed for all users. Change it in Task Manager > Startup apps (needs admin rights).');
      item.enabled = Boolean(args.enabled);
      return s.startup;
    }
    case 'add_startup': {
      const name = String(args.name).trim();
      const cmd = String(args.command).trim();
      if (!name || !cmd) throw new Error('Name and program path cannot be empty.');
      if (s.startup.some((i) => i.name.toLowerCase() === name.toLowerCase())) throw new Error(`A startup entry named "${name}" already exists.`);
      s.startup.push({ id: `user-run:${name}`, name, command: cmd, source: 'userRun', enabled: true, editable: true });
      s.startup.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
      return s.startup;
    }
    case 'remove_startup':
      s.startup = s.startup.filter((i) => i.id !== args.id || !i.editable);
      return s.startup;
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
    case 'list_tweaks':
      return s.tweaks;
    case 'set_tweak': {
      const t = s.tweaks.find((x) => x.id === args.id);
      if (!t) throw new Error('Not found');
      t.enabled = Boolean(args.enabled);
      return s.tweaks;
    }
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
