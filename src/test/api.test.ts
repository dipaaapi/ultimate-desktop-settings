import { describe, expect, it } from 'vitest';
import { api } from '../lib/api';

describe('api in the browser preview', () => {
  it('answers from sample data', async () => {
    const info = await api.appInfo();
    expect(info.version).toBe('2.0.0');
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
