import { ArrowClockwise20Regular, Broom20Regular, Delete20Regular, FolderOpen20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { formatBytes } from '../lib/format';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import { Button, ConfirmButton, ErrorBox, Loading, Note, Row, Section, Stat } from '../components/ui';

export default function QuickActions() {
  const overview = useAsync(api.overview);
  const { busy, run } = useAction();
  const o = overview.data;

  return (
    <>
      <Section title="At a glance">
        {overview.error ? (
          <ErrorBox message={overview.error} onRetry={overview.reload} />
        ) : !o ? (
          <Loading rows={1} />
        ) : (
          <div className="stats">
            <Stat label="Startup apps on" value={`${o.startupEnabled} / ${o.startupTotal}`} />
            <Stat label="Temp files" value={formatBytes(o.temp.bytes)} hint={`${o.temp.items} items`} />
            <Stat label="Recycle Bin" value={formatBytes(o.recycleBin.bytes)} hint={`${o.recycleBin.items} items`} />
            <Stat label="Paired Bluetooth devices" value={o.bluetoothDevices} />
          </div>
        )}
      </Section>

      <Section title="Actions">
        <div className="list">
          <Row title="Refresh the desktop" description="Redraws desktop icons and tells Windows that settings changed.">
            <Button
              icon={<ArrowClockwise20Regular />}
              disabled={busy === 'refresh'}
              onClick={() => run('refresh', api.refreshDesktop, 'Desktop refreshed.')}
            >
              Refresh
            </Button>
          </Row>
          <Row
            title="Clean temporary files"
            description="Deletes files in your Temp folder. Files changed in the last 24 hours and files in use are kept."
          >
            <Button size="sm" variant="subtle" icon={<FolderOpen20Regular />} onClick={() => run('open-temp', () => api.openShortcut('temp-folder'))}>
              Open
            </Button>
            <ConfirmButton
              icon={<Broom20Regular />}
              disabled={busy === 'clean' || o?.temp.items === 0}
              confirmLabel="Delete temp files?"
              onConfirm={async () => {
                await run('clean', api.cleanTemp, (r) => `Freed ${formatBytes(r.freedBytes)} (${r.removed} removed, ${r.skipped} kept).`);
                overview.reload();
              }}
            >
              Clean
            </ConfirmButton>
          </Row>
          <Row title="Empty the Recycle Bin" description="Permanently deletes everything in the Recycle Bin on all drives.">
            <Button size="sm" variant="subtle" icon={<FolderOpen20Regular />} onClick={() => run('open-bin', () => api.openShortcut('recycle-bin'))}>
              Open
            </Button>
            <ConfirmButton
              icon={<Delete20Regular />}
              disabled={busy === 'bin' || o?.recycleBin.items === 0}
              confirmLabel="Empty it for good?"
              onConfirm={async () => {
                await run('bin', api.emptyRecycleBin, (r) => `Recycle Bin emptied (${formatBytes(r.bytes)} freed).`);
                overview.reload();
              }}
            >
              Empty
            </ConfirmButton>
          </Row>
        </div>
        <Note>Destructive actions ask for a second click before they run.</Note>
      </Section>
    </>
  );
}
