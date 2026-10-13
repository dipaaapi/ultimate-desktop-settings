import { Add20Regular, ArrowSwap20Regular, Open20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { ServiceState } from '../lib/types';
import { Badge, Button, ErrorBox, Loading, Note, Row, Section } from '../components/ui';

const LABELS: Record<ServiceState, string> = {
  running: 'Running',
  stopped: 'Stopped',
  starting: 'Starting…',
  stopping: 'Stopping…',
  unknown: 'Not available',
};

export default function Bluetooth() {
  const bt = useAsync(api.bluetoothState);
  const { busy, run } = useAction();

  if (bt.error) return <ErrorBox message={bt.error} onRetry={bt.reload} />;
  if (!bt.data) return <Loading rows={3} />;
  const { service, devices } = bt.data;
  const running = service === 'running';

  const control = async (start: boolean) => {
    await run('svc', () => api.controlBluetooth(start), start ? 'Bluetooth service starting.' : 'Bluetooth service stopping.');
    setTimeout(bt.reload, 1200);
  };

  return (
    <>
      <Section title="Bluetooth service">
        <div className="list">
          <Row
            title={<>Bluetooth Support Service <Badge tone={running ? 'good' : 'warn'}>{LABELS[service]}</Badge></>}
            description="Windows asks for permission before the service is started or stopped."
          >
            {service !== 'unknown' && (
              <Button variant={running ? 'default' : 'primary'} disabled={busy === 'svc'} onClick={() => control(!running)}>
                {running ? 'Stop' : 'Start'}
              </Button>
            )}
          </Row>
        </div>
      </Section>

      <Section
        title="Paired devices"
        actions={
          <>
            <Button size="sm" icon={<Add20Regular />} onClick={() => run('pair', () => api.openShortcut('bluetooth-pair'))}>
              Add device
            </Button>
            <Button size="sm" icon={<ArrowSwap20Regular />} onClick={() => run('files', () => api.openShortcut('bluetooth-files'))}>
              Send or receive files
            </Button>
          </>
        }
      >
        {devices.length === 0 ? (
          <div className="empty">No paired devices yet.</div>
        ) : (
          <div className="list">
            {devices.map((d) => (
              <Row key={d.address} title={d.name} description={<span className="mono">{d.address}</span>} />
            ))}
          </div>
        )}
        <div className="section-actions" style={{ marginTop: 10 }}>
          <Button size="sm" variant="subtle" icon={<Open20Regular />} onClick={() => run('set', () => api.openShortcut('bluetooth'))}>
            Bluetooth settings
          </Button>
        </div>
        <Note>Turning the radio on or off is done in Windows Settings; this board manages the service and shows what is paired.</Note>
      </Section>
    </>
  );
}
