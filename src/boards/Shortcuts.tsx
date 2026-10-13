import { useMemo, useState } from 'react';
import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { ShortcutInfo } from '../lib/types';
import { ErrorBox, Loading, Section, TextInput } from '../components/ui';

export default function Shortcuts() {
  const shortcuts = useAsync(api.listShortcuts);
  const { run } = useAction();
  const [query, setQuery] = useState('');

  const groups = useMemo(() => {
    const q = query.trim().toLowerCase();
    const map = new Map<string, ShortcutInfo[]>();
    for (const s of shortcuts.data ?? []) {
      if (q && !`${s.title} ${s.description}`.toLowerCase().includes(q)) continue;
      map.set(s.group, [...(map.get(s.group) ?? []), s]);
    }
    return [...map];
  }, [shortcuts.data, query]);

  if (shortcuts.error) return <ErrorBox message={shortcuts.error} onRetry={shortcuts.reload} />;
  if (!shortcuts.data) return <Loading rows={4} />;

  return (
    <>
      <div className="form-row" style={{ marginBottom: 18 }}>
        <TextInput label="Search" placeholder="Bluetooth, display, Task Manager…" value={query} onChange={(e) => setQuery(e.target.value)} />
      </div>
      {groups.length === 0 && <div className="empty">Nothing matches “{query}”.</div>}
      {groups.map(([group, items]) => (
        <Section key={group} title={group}>
          <div className="grid">
            {items.map((s) => (
              <button key={s.id} type="button" className="tile" onClick={() => run(s.id, () => api.openShortcut(s.id), `Opening ${s.title}…`)}>
                <span className="tile-title">{s.title}</span>
                <span className="tile-desc">{s.description}</span>
              </button>
            ))}
          </div>
        </Section>
      ))}
    </>
  );
}
