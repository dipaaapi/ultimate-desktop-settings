import { useState } from 'react';
import { Open20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { usePreferences } from '../lib/theme';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import { Badge, Button, ErrorBox, Loading, Note, Row, Section, Slider, Switch } from '../components/ui';

export default function Display() {
  const displays = useAsync(api.listDisplays);
  const { preferences, update } = usePreferences();
  const { run } = useAction();
  const saved = preferences?.nightLightStrength ?? 40;
  const [strength, setStrength] = useState(saved);
  const [shown, setShown] = useState(saved);
  if (shown !== saved) {
    setShown(saved);
    setStrength(saved);
  }

  return (
    <>
      <Section title="Night Light">
        <div className="list">
          <Row title="Warm screen filter" description="Tints every monitor warmer to reduce blue light. It turns off when you exit the app.">
            <Switch
              label="Night Light"
              checked={preferences?.nightLightEnabled ?? false}
              disabled={!preferences}
              onChange={(value) => preferences && run('nl', () => update({ ...preferences, nightLightEnabled: value }))}
            />
          </Row>
          <Row title="Strength" description={`${strength}%`}>
            <div style={{ width: 'min(260px, 60vw)' }}>
              <Slider
                label="Night Light strength"
                value={strength}
                disabled={!preferences}
                onChange={setStrength}
                onCommit={(value) => preferences && run('nl', () => update({ ...preferences, nightLightStrength: value }))}
              />
            </div>
          </Row>
        </div>
        <div className="section-actions" style={{ marginTop: 10 }}>
          <Button size="sm" variant="subtle" icon={<Open20Regular />} onClick={() => run('ws', () => api.openShortcut('night-light'))}>
            Windows night light schedule
          </Button>
        </div>
      </Section>

      <Section
        title="Monitors"
        actions={
          <Button size="sm" variant="subtle" icon={<Open20Regular />} onClick={() => run('ds', () => api.openShortcut('display'))}>
            Display settings
          </Button>
        }
      >
        {displays.error ? (
          <ErrorBox message={displays.error} onRetry={displays.reload} />
        ) : !displays.data ? (
          <Loading rows={2} />
        ) : (
          <div className="list">
            {displays.data.map((d) => (
              <Row
                key={d.device}
                title={
                  <>
                    {d.name}
                    {d.primary && <Badge tone="accent">Main display</Badge>}
                  </>
                }
                description={`${d.width} × ${d.height} · ${d.refreshHz} Hz · ${d.scalePercent}% scale`}
              />
            ))}
          </div>
        )}
        <Note>Resolution and scale are changed in Windows Settings so Windows can confirm them safely.</Note>
      </Section>
    </>
  );
}
