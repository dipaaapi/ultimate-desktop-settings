import { Open20Regular, Globe20Regular, EyeOff20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import { Badge, Button, ErrorBox, Loading, Note, Row, Section } from '../components/ui';

export default function Browsers() {
  const browsers = useAsync(api.listBrowsers);
  const { busy, run } = useAction();

  if (browsers.error) return <ErrorBox message={browsers.error} onRetry={browsers.reload} />;
  if (!browsers.data) return <Loading rows={3} />;

  return (
    <Section
      title={`Installed browsers (${browsers.data.length})`}
      actions={
        <Button size="sm" variant="subtle" icon={<Open20Regular />} onClick={() => run('d', () => api.openShortcut('default-apps'))}>
          Change default
        </Button>
      }
    >
      {browsers.data.length === 0 ? (
        <div className="empty">No browsers were found.</div>
      ) : (
        <div className="list">
          {browsers.data.map((b) => (
            <Row key={b.id} title={<>{b.name}{b.isDefault && <Badge tone="good">Default</Badge>}</>} description={<span className="mono">{b.exe}</span>}>
              {b.supportsPrivate && (
                <Button size="sm" icon={<EyeOff20Regular />} disabled={busy === `${b.id}-p`} onClick={() => run(`${b.id}-p`, () => api.launchBrowser(b.id, true))}>
                  Private
                </Button>
              )}
              <Button size="sm" variant="primary" icon={<Globe20Regular />} disabled={busy === b.id} onClick={() => run(b.id, () => api.launchBrowser(b.id, false))}>
                Open
              </Button>
            </Row>
          ))}
        </div>
      )}
      <Note>Windows only lets you pick the default browser in its own Settings, which the button above opens.</Note>
    </Section>
  );
}
