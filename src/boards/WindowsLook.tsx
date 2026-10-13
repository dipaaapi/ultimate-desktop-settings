import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { TweakState } from '../lib/types';
import { ErrorBox, Loading, Note, Row, Section, Switch } from '../components/ui';

export default function WindowsLook() {
  const tweaks = useAsync(api.listTweaks);
  const { busy, run } = useAction();

  if (tweaks.error) return <ErrorBox message={tweaks.error} onRetry={tweaks.reload} />;
  if (!tweaks.data) return <Loading rows={5} />;

  const groups = new Map<string, TweakState[]>();
  for (const t of tweaks.data) groups.set(t.group, [...(groups.get(t.group) ?? []), t]);

  return (
    <>
      {[...groups].map(([group, items]) => (
        <Section key={group} title={group}>
          <div className="list">
            {items.map((t) => (
              <Row key={t.id} title={t.title} description={t.description}>
                <Switch
                  label={t.title}
                  checked={t.enabled}
                  disabled={busy === t.id}
                  onChange={async (value) => {
                    const next = await run(t.id, () => api.setTweak(t.id, value));
                    if (next) tweaks.set(next);
                  }}
                />
              </Row>
            ))}
          </div>
        </Section>
      ))}
      <Note>Changes apply to your Windows account only. Some taskbar options need Windows 11 and may take a moment to appear.</Note>
    </>
  );
}
