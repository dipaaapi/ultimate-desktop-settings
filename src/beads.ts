import { lazy, type ComponentType, type LazyExoticComponent } from 'react';
import {
  AppsList24Regular,
  BatteryCharge24Regular,
  Bluetooth24Regular,
  Clock24Regular,
  Flash24Regular,
  FolderOpen24Regular,
  Globe24Regular,
  Grid24Regular,
  Info24Regular,
  Keyboard24Regular,
  Note24Regular,
  PaintBrush24Regular,
  Rocket24Regular,
  Settings24Regular,
  ShieldLock24Regular,
  Speaker224Regular,
  WeatherMoon24Regular,
  Window24Regular,
  WindowWrench24Regular,
  type FluentIcon,
} from '@fluentui/react-icons';

/** Bead colors match the logo's beads. */
export type Category = 'desktop' | 'devices' | 'apps' | 'utilities';

export const CATEGORIES: Record<Category, { label: string; color: string }> = {
  desktop: { label: 'Desktop', color: '#5EEAD4' },
  devices: { label: 'Devices', color: '#60A5FA' },
  apps: { label: 'Apps', color: '#C084FC' },
  utilities: { label: 'Utilities', color: '#FCD34D' },
};

export interface BeadDef {
  id: string;
  title: string;
  subtitle: string;
  category: Category;
  icon: FluentIcon;
  Board: LazyExoticComponent<ComponentType>;
}

const board = (load: () => Promise<{ default: ComponentType }>) => lazy(load);

/** Beads to the left of the title badge, in order from left to right. */
export const LEFT_BEADS: BeadDef[] = [
  { id: 'quick', title: 'Quick Actions', subtitle: 'Refresh the desktop, clean temp files and the Recycle Bin', category: 'desktop', icon: Flash24Regular, Board: board(() => import('./boards/QuickActions')) },
  { id: 'look', title: 'Windows Look', subtitle: 'Wallpaper, dark mode, colors, desktop icons and effects', category: 'desktop', icon: PaintBrush24Regular, Board: board(() => import('./boards/WindowsLook')) },
  { id: 'taskbar', title: 'Taskbar & Start', subtitle: 'Alignment, buttons, search, auto-hide and Start menu', category: 'desktop', icon: Window24Regular, Board: board(() => import('./boards/Taskbar')) },
  { id: 'explorer', title: 'File Explorer', subtitle: 'Extensions, hidden files, Home, navigation and the classic menu', category: 'desktop', icon: FolderOpen24Regular, Board: board(() => import('./boards/FileExplorer')) },
  { id: 'system', title: 'System & Multitasking', subtitle: 'Snap, notifications, clipboard, gaming, storage and clock format', category: 'desktop', icon: WindowWrench24Regular, Board: board(() => import('./boards/SystemSettings')) },
  { id: 'privacy', title: 'Privacy & Search', subtitle: 'App permissions, ads, tips and search', category: 'desktop', icon: ShieldLock24Regular, Board: board(() => import('./boards/Privacy')) },
  { id: 'display', title: 'Display & Night Light', subtitle: 'Your monitors and a warm screen filter', category: 'desktop', icon: WeatherMoon24Regular, Board: board(() => import('./boards/Display')) },
  { id: 'shortcuts', title: 'Settings Shortcuts', subtitle: 'Jump to any Windows settings page or tool', category: 'desktop', icon: Grid24Regular, Board: board(() => import('./boards/Shortcuts')) },
  { id: 'sound', title: 'Sound', subtitle: 'Master volume, per-app mixer and output devices', category: 'devices', icon: Speaker224Regular, Board: board(() => import('./boards/Sound')) },
  { id: 'bluetooth', title: 'Bluetooth', subtitle: 'Bluetooth service and paired devices', category: 'devices', icon: Bluetooth24Regular, Board: board(() => import('./boards/Bluetooth')) },
  { id: 'input', title: 'Mouse & Keyboard', subtitle: 'Pointer speed, buttons, scrolling and key repeat', category: 'devices', icon: Keyboard24Regular, Board: board(() => import('./boards/MouseKeyboard')) },
  { id: 'power', title: 'Power & Battery', subtitle: 'Power plan, brightness, screen and sleep, lid and battery saver', category: 'devices', icon: BatteryCharge24Regular, Board: board(() => import('./boards/Power')) },
];

/** Beads to the right of the title badge, in order from left to right. */
export const RIGHT_BEADS: BeadDef[] = [
  { id: 'startup', title: 'Startup Apps', subtitle: 'Choose what starts when you sign in', category: 'apps', icon: Rocket24Regular, Board: board(() => import('./boards/StartupApps')) },
  { id: 'apps', title: 'Installed Apps', subtitle: 'Find, launch and close your programs', category: 'apps', icon: AppsList24Regular, Board: board(() => import('./boards/InstalledApps')) },
  { id: 'browsers', title: 'Browsers', subtitle: 'Your browsers, the default one and private windows', category: 'apps', icon: Globe24Regular, Board: board(() => import('./boards/Browsers')) },
  { id: 'time', title: 'Time Center', subtitle: 'Timer, alarms, stopwatch and world clock', category: 'utilities', icon: Clock24Regular, Board: board(() => import('./boards/TimeCenter')) },
  { id: 'notes', title: 'Notes', subtitle: 'Scrapbook notes saved on this PC', category: 'utilities', icon: Note24Regular, Board: board(() => import('./boards/Notes')) },
  { id: 'prefs', title: 'Preferences', subtitle: 'App size, theme, tray and launch at startup', category: 'utilities', icon: Settings24Regular, Board: board(() => import('./boards/Preferences')) },
  { id: 'about', title: 'About & Credits', subtitle: 'Version, developer and the tech stack', category: 'utilities', icon: Info24Regular, Board: board(() => import('./boards/About')) },
];

export const ALL_BEADS = [...LEFT_BEADS, ...RIGHT_BEADS];

export const findBead = (id: string) => ALL_BEADS.find((b) => b.id === id);
