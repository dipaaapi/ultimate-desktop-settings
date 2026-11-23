import { Battery020Regular, BatteryCharge20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { BatteryStatus } from '../lib/types';
import { SettingsList } from '../components/SettingsList';
import { Badge, ErrorBox, Loading, Note, Row, Section } from '../components/ui';

function batteryText(b: BatteryStatus) {
  if (!b.hasBattery) return 'This PC runs on mains power.';
  const parts = [b.percent === null ? 'Battery' : `${b.percent}%`, b.pluggedIn ? (b.charging ? 'charging' : 'plugged in') : 'on battery'];
  if (b.saverOn) parts.push('battery saver on');
  return parts.join(' · ');
}

export default function Power() {
  const info = useAsync(api.powerInfo);
  const { busy, run } = useAction();

  const header = info.error ? (
    <ErrorBox message={info.error} onRetry={info.reload} />
  ) : !info.data ? (
    <Loading rows={2} />
  ) : (
    <>
      <Section title="Battery">
        <div className="list">
          <Row title={info.data.battery.pluggedIn ? 'Plugged in' : 'On battery'} description={batteryText(info.data.battery)}>
            {info.data.battery.pluggedIn ? <BatteryCharge20Regular aria-hidden="true" /> : <Battery020Regular aria-hidden="true" />}
          </Row>
        </div>
      </Section>
      <Section title="Power plan">
        <div className="list">
          {info.data.plans.map((p) => (
            <Row key={p.id} title={<>{p.name || 'Unnamed plan'}{p.active && <Badge tone="accent">Active</Badge>}</>}>
              <button
                type="button"
                role="radio"
                aria-checked={p.active}
                aria-label={`Use ${p.name}`}
                className={`radio ${p.active ? 'on' : ''}`}
                disabled={p.active || busy === 'plan'}
                onClick={async () => {
                  const next = await run('plan', () => api.setPowerPlan(p.id), `${p.name} is now active.`);
                  if (next) info.set(next);
                }}
              />
            </Row>
          ))}
        </div>
        <Note>The settings below belong to the active plan.</Note>
      </Section>
    </>
  );

  return <SettingsList page="power" before={header} />;
}
