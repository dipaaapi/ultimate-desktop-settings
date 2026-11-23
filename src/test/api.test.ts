import { describe, expect, it } from 'vitest';
import { api } from '../lib/api';

describe('api in the browser preview', () => {
  it('answers from sample data', async () => {
    const info = await api.appInfo();
    expect(info.version).toBe('2.1.0');
    expect((await api.listBrowsers()).some((b) => b.isDefault)).toBe(true);
  });

  it('toggles a startup entry', async () => {
    const before = await api.listStartup();
    const item = before.find((i) => i.editable)!;
    const after = await api.setStartupEnabled(item.id, !item.enabled);
    expect(after.find((i) => i.id === item.id)!.enabled).toBe(!item.enabled);
  });

  it('refuses to delete a pinned note', async () => {
    const pinned = (await api.listNotes()).find((n) => n.pinned)!;
    await expect(api.deleteNote(pinned.title)).rejects.toThrow();
  });

  it('rejects note titles Windows cannot store', async () => {
    await expect(api.createNote('a/b')).rejects.toThrow(/cannot contain/);
  });
});

describe('settings in the browser preview', () => {
  it('lists every board and changes a switch, a choice and a slider', async () => {
    for (const page of ['look', 'taskbar', 'explorer', 'system', 'privacy', 'input', 'power'] as const) {
      expect((await api.listSettings(page)).length).toBeGreaterThan(5);
    }
    let taskbar = await api.listSettings('taskbar');
    const search = taskbar.find((t) => t.id === 'taskbar-search')!;
    taskbar = await api.setSetting(search.id, 0);
    expect(taskbar.find((t) => t.id === 'taskbar-search')!.selected).toBe(0);

    const explorer = await api.setSetting('file-extensions', 1);
    expect(explorer.find((t) => t.id === 'file-extensions')!.enabled).toBe(true);

    const input = await api.setSetting('mouse-speed', 14);
    expect(input.find((t) => t.id === 'mouse-speed')!.value).toBe(14);
    await expect(api.setSetting('mouse-speed', 99)).rejects.toThrow();
    await expect(api.setSetting('not-a-setting', 1)).rejects.toThrow();
  });

  it('switches the power plan', async () => {
    const info = await api.powerInfo();
    const other = info.plans.find((p) => !p.active)!;
    const next = await api.setPowerPlan(other.id);
    expect(next.plans.find((p) => p.active)!.id).toBe(other.id);
  });
});

describe('startup in the browser preview', () => {
  it('shows entries Task Manager hides and keeps read-only ones safe', async () => {
    const items = await api.listStartup();
    for (const source of ['machineRun32', 'commonFolder', 'storeApp', 'scheduledTask', 'userRunOnce'] as const) {
      expect(items.some((i) => i.source === source)).toBe(true);
    }
    const task = items.find((i) => i.source === 'scheduledTask')!;
    await expect(api.removeStartup(task.id)).rejects.toThrow();
    const once = items.find((i) => i.source === 'userRunOnce')!;
    await expect(api.setStartupEnabled(once.id, false)).rejects.toThrow();
  });
});
