import { useState } from 'react';
import { Add20Regular, ArrowClockwise20Regular, Delete20Regular, Folder20Regular, Search20Regular } from '@fluentui/react-icons';
import { api, pickExecutable } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { StartupItem, StartupSource } from '../lib/types';
import { Badge, Button, ConfirmButton, ErrorBox, Loading, Note, Row, Section, Switch, TextInput } from '../components/ui';

const SOURCE: Record<StartupSource, string> = {
  userRun: 'Your account',
  userRunOnce: 'Runs once',
  machineRun: 'All users',
  machineRun32: 'All users (32-bit)',
  machineRunOnce: 'Runs once, all users',
  userPolicy: 'Group policy',
  machinePolicy: 'Group policy, all users',
  userFolder: 'Startup folder',
  commonFolder: 'All users Startup folder',
  storeApp: 'Store app',
  scheduledTask: 'Scheduled task',
};

/** Where each kind of entry is listed. Task Manager hides the last two groups. */
const GROUPS: { title: string; sources: StartupSource[] }[] = [
  { title: 'Programs', sources: ['userRun', 'machineRun', 'machineRun32', 'userFolder', 'commonFolder'] },
  { title: 'Store apps', sources: ['storeApp'] },
  { title: 'Scheduled tasks at sign-in', sources: ['scheduledTask'] },
  { title: 'Run once and group policy', sources: ['userRunOnce', 'machineRunOnce', 'userPolicy', 'machinePolicy'] },
];

type Filter = 'all' | 'on' | 'off';

export default function StartupApps() {
  const items = useAsync(api.listStartup);
  const { busy, run } = useAction();
  const [name, setName] = useState('');
  const [command, setCommand] = useState('');
  const [filter, setFilter] = useState<Filter>('all');
  const [query, setQuery] = useState('');

  const apply = async (key: string, action: () => Promise<StartupItem[]>, success?: string) => {
    const next = await run(key, action, success);
    if (next) items.set(next);
    return next;
  };

  if (items.error) return <ErrorBox message={items.error} onRetry={items.reload} />;
  if (!items.data) return <Loading rows={4} />;

  const renderItem = (item: StartupItem) => (
    <Row
      key={item.id}
      title={
        <>
          {item.name}
          <Badge>{SOURCE[item.source]}</Badge>
          {item.needsAdmin && (item.editable || item.removable) && <Badge tone="accent">Admin</Badge>}
          {!item.editable && !item.removable && <Badge tone="warn">Read-only</Badge>}
        </>
      }
      description={
        <span className="mono" title={item.location}>
          {item.command}
        </span>
      }
    >
      {item.removable && (
        <ConfirmButton
          size="sm"
          variant="subtle"
          aria-label={`Remove ${item.name}`}
          icon={<Delete20Regular />}
          confirmLabel="Remove?"
          disabled={busy === item.id}
          onConfirm={() => apply(item.id, () => api.removeStartup(item.id), `${item.name} removed.`)}
        />
      )}
      {item.editable && (
        <Switch
          label={`Start ${item.name} at sign-in`}
          checked={item.enabled}
          disabled={busy === item.id}
          onChange={(v) => apply(item.id, () => api.setStartupEnabled(item.id, v))}
        />
      )}
    </Row>
  );

  const q = query.trim().toLowerCase();
  const visible = items.data.filter(
    (i) => (filter === 'all' || (filter === 'on') === i.enabled) && (!q || `${i.name} ${i.command}`.toLowerCase().includes(q)),
  );
  const onCount = items.data.filter((i) => i.enabled).length;

  return (
    <>
      <div className="startup-toolbar">
        <div className="tabs tabs-sm" role="radiogroup" aria-label="Show">
          {(
            [
              ['all', `All (${items.data.length})`],
              ['on', `On (${onCount})`],
              ['off', `Off (${items.data.length - onCount})`],
            ] as const
          ).map(([id, label]) => (
            <button key={id} type="button" role="radio" className="tab" aria-checked={filter === id} aria-selected={filter === id} onClick={() => setFilter(id)}>
              {label}
            </button>
          ))}
        </div>
        <label className="settings-search">
          <Search20Regular aria-hidden="true" />
          <input className="input" type="search" placeholder="Find an app" aria-label="Find a startup app" value={query} onChange={(e) => setQuery(e.target.value)} />
        </label>
        <Button size="sm" variant="subtle" icon={<ArrowClockwise20Regular />} onClick={items.reload} disabled={items.loading}>
          Refresh
        </Button>
      </div>

      {GROUPS.map((group) => {
        const list = visible.filter((i) => group.sources.includes(i.source));
        if (!list.length) return null;
        return (
          <Section key={group.title} title={`${group.title} (${list.length})`}>
            <div className="list">{list.map(renderItem)}</div>
          </Section>
        );
      })}
      {!visible.length && <div className="empty">No startup entries match.</div>}

      <Note>
        Every place Windows starts programs from is listed, including 32-bit and all-users entries, Store apps and scheduled tasks that Task Manager doesn't show. Turning
        an entry off uses the same switch as Task Manager. Entries marked Admin ask for permission first.
      </Note>

      <Section title="Add a program">
        <form
          className="card form-row"
          onSubmit={async (e) => {
            e.preventDefault();
            const next = await apply('add', () => api.addStartup(name, command), `${name.trim()} will start at sign-in.`);
            if (next) {
              setName('');
              setCommand('');
            }
          }}
        >
          <TextInput label="Name" value={name} onChange={(e) => setName(e.target.value)} placeholder="My app" />
          <TextInput label="Program" value={command} onChange={(e) => setCommand(e.target.value)} placeholder="C:\Path\to\app.exe" />
          <Button
            icon={<Folder20Regular />}
            onClick={async () => {
              const picked = await pickExecutable();
              if (!picked) return;
              setCommand(picked);
              if (!name) setName(picked.split(/[\\/]/).pop()?.replace(/\.[^.]+$/, '') ?? '');
            }}
          >
            Browse
          </Button>
          <button type="submit" className="btn btn-primary" disabled={!name.trim() || !command.trim() || busy === 'add'}>
            <span className="btn-icon">
              <Add20Regular />
            </span>
            Add
          </button>
        </form>
      </Section>
    </>
  );
}
