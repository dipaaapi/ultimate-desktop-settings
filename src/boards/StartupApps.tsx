import { useState } from 'react';
import { Add20Regular, Delete20Regular, Folder20Regular } from '@fluentui/react-icons';
import { api, pickExecutable } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { StartupItem, StartupSource } from '../lib/types';
import { Badge, Button, ConfirmButton, ErrorBox, Loading, Note, Row, Section, Switch, TextInput } from '../components/ui';

const SOURCE: Record<StartupSource, string> = {
  userRun: 'Your account',
  machineRun: 'All users',
  userFolder: 'Startup folder',
};

export default function StartupApps() {
  const items = useAsync(api.listStartup);
  const { busy, run } = useAction();
  const [name, setName] = useState('');
  const [command, setCommand] = useState('');

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
          {!item.editable && <Badge tone="warn">Read-only</Badge>}
        </>
      }
      description={<span className="mono">{item.command}</span>}
    >
      {item.editable && item.source !== 'machineRun' && (
        <ConfirmButton
          size="sm"
          variant="subtle"
          aria-label={`Remove ${item.name}`}
          icon={<Delete20Regular />}
          confirmLabel="Remove?"
          onConfirm={() => apply(item.id, () => api.removeStartup(item.id), `${item.name} removed.`)}
        />
      )}
      <Switch
        label={`Start ${item.name} at sign-in`}
        checked={item.enabled}
        disabled={!item.editable || busy === item.id}
        onChange={(v) => apply(item.id, () => api.setStartupEnabled(item.id, v))}
      />
    </Row>
  );

  const active = items.data.filter((i) => i.enabled);
  const disabled = items.data.filter((i) => !i.enabled);

  return (
    <>
      <Section title={`Starts at sign-in (${active.length})`}>
        {active.length ? <div className="list">{active.map(renderItem)}</div> : <div className="empty">Nothing starts with Windows.</div>}
      </Section>
      <Section title={`Turned off (${disabled.length})`}>
        {disabled.length ? <div className="list">{disabled.map(renderItem)}</div> : <div className="empty">No turned-off entries.</div>}
        <Note>Turned-off entries use the same switch as Task Manager, so Windows and this app always agree.</Note>
      </Section>
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
            <span className="btn-icon"><Add20Regular /></span>
            Add
          </button>
        </form>
      </Section>
    </>
  );
}
