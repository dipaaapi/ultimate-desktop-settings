import { useMemo, useState } from 'react';
import { Dismiss20Regular, FolderOpen20Regular, Play20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { AppCategory } from '../lib/types';
import { Button, ErrorBox, Loading, Note, Row, TextInput } from '../components/ui';

const CATEGORIES: (AppCategory | 'All')[] = ['All', 'Development', 'Design & Creative', 'Gaming', 'Office & General Use'];

export default function InstalledApps() {
  const apps = useAsync(api.listApps);
  const { busy, run } = useAction();
  const [query, setQuery] = useState('');
  const [category, setCategory] = useState<AppCategory | 'All'>('All');

  const shown = useMemo(() => {
    const q = query.trim().toLowerCase();
    return (apps.data ?? []).filter(
      (a) => (category === 'All' || a.category === category) && (!q || `${a.name} ${a.publisher ?? ''}`.toLowerCase().includes(q)),
    );
  }, [apps.data, query, category]);

  if (apps.error) return <ErrorBox message={apps.error} onRetry={apps.reload} />;
  if (!apps.data) return <Loading rows={5} />;

  return (
    <>
      <div className="tabs" role="tablist" aria-label="Categories">
        {CATEGORIES.map((c) => (
          <button key={c} type="button" role="tab" className="tab" aria-selected={category === c} onClick={() => setCategory(c)}>
            {c === 'All' ? `All (${apps.data!.length})` : c}
          </button>
        ))}
      </div>
      <div className="form-row" style={{ marginBottom: 14 }}>
        <TextInput label="Search" placeholder="App or publisher" value={query} onChange={(e) => setQuery(e.target.value)} />
      </div>
      {shown.length === 0 ? (
        <div className="empty">No apps match.</div>
      ) : (
        <div className="list">
          {shown.map((a) => (
            <Row key={a.id} title={a.name} description={[a.publisher, a.version && `v${a.version}`, a.category].filter(Boolean).join(' · ')}>
              <Button size="sm" variant="primary" icon={<Play20Regular />} disabled={!a.exe || busy === a.id} onClick={() => run(a.id, () => api.launchApp(a.id), `Opening ${a.name}…`)}>
                Open
              </Button>
              <Button size="sm" aria-label={`Open ${a.name} folder`} icon={<FolderOpen20Regular />} disabled={!a.installLocation} onClick={() => run(`${a.id}-f`, () => api.openAppFolder(a.id))} />
              <Button
                size="sm"
                aria-label={`Close ${a.name}`}
                icon={<Dismiss20Regular />}
                disabled={!a.installLocation}
                onClick={() => run(`${a.id}-c`, () => api.closeApp(a.id), (n) => (n ? `Asked ${n} window(s) of ${a.name} to close.` : `${a.name} is not running.`))}
              />
            </Row>
          ))}
        </div>
      )}
      <Note>Close asks the app's windows to close normally, so it can still offer to save your work. Uninstalling is done from Windows Settings.</Note>
    </>
  );
}
