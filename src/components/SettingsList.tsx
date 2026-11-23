import { useState, type ReactNode } from 'react';
import { ArrowClockwise20Regular, Search20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { SettingsPage, TweakState } from '../lib/types';
import { Badge, Button, ErrorBox, Loading, Note, Row, Section, Slider, Switch } from './ui';

/** Choices with this many short options show as buttons; longer ones use a menu. */
const MAX_BUTTONS = 4;

/**
 * Every Windows setting on one board, grouped as Windows groups them.
 * Switches, choices and sliders all go through `api.setSetting`.
 */
export function SettingsList({ page, before, after }: { page: SettingsPage; before?: ReactNode; after?: ReactNode }) {
  const settings = useAsync(() => api.listSettings(page), [page]);
  const { busy, run } = useAction();
  const [query, setQuery] = useState('');

  if (settings.error) return <ErrorBox message={settings.error} onRetry={settings.reload} />;
  if (!settings.data) return <Loading rows={5} />;

  const apply = async (t: TweakState, value: number) => {
    const next = await run(t.id, () => api.setSetting(t.id, value));
    if (next) settings.set(next);
  };

  const q = query.trim().toLowerCase();
  const shown = settings.data.filter((t) => !q || `${t.title} ${t.description} ${t.group}`.toLowerCase().includes(q));
  const groups = new Map<string, TweakState[]>();
  for (const t of shown) groups.set(t.group, [...(groups.get(t.group) ?? []), t]);
  const needsRestart = settings.data.some((t) => t.needsRestart);

  return (
    <>
      {settings.data.length > 8 && (
        <label className="settings-search">
          <Search20Regular aria-hidden="true" />
          <input className="input" type="search" placeholder="Find a setting" aria-label="Find a setting" value={query} onChange={(e) => setQuery(e.target.value)} />
        </label>
      )}
      {before}
      {[...groups].map(([group, items]) => (
        <Section key={group} title={group}>
          <div className="list">
            {items.map((t) => (
              <Row
                key={t.id}
                title={
                  <>
                    {t.title}
                    {t.needsRestart && <Badge tone="warn">Restart Explorer</Badge>}
                  </>
                }
                description={t.description}
              >
                <SettingControl setting={t} disabled={busy === t.id} onChange={(v) => apply(t, v)} />
              </Row>
            ))}
          </div>
        </Section>
      ))}
      {!shown.length && <div className="empty">No setting matches “{query}”.</div>}
      {after}
      {needsRestart && (
        <Section
          title="Explorer"
          actions={
            <Button size="sm" icon={<ArrowClockwise20Regular />} disabled={busy === 'explorer'} onClick={() => run('explorer', api.restartExplorer, 'Explorer restarted.')}>
              Restart Explorer
            </Button>
          }
        >
          <Note>Settings marked “Restart Explorer” apply after Explorer restarts. Open folder windows close; your apps keep running.</Note>
        </Section>
      )}
      <Note>Changes apply to your Windows account right away. Some options need Windows 11 or a recent update.</Note>
    </>
  );
}

function SettingControl({ setting: t, disabled, onChange }: { setting: TweakState; disabled: boolean; onChange: (value: number) => void }) {
  if (t.range) return <RangeControl setting={t} disabled={disabled} onCommit={onChange} />;
  if (t.toggle) return <Switch label={t.title} checked={t.enabled} disabled={disabled || t.selected === null} onChange={(v) => onChange(v ? 1 : 0)} />;
  const short = t.options.length <= MAX_BUTTONS && t.options.every((o) => o.length <= 16);
  if (short) {
    return (
      <div className="tabs tabs-sm" role="radiogroup" aria-label={t.title}>
        {t.options.map((label, i) => (
          <button key={label} type="button" role="radio" className="tab" aria-checked={t.selected === i} aria-selected={t.selected === i} disabled={disabled} onClick={() => t.selected !== i && onChange(i)}>
            {label}
          </button>
        ))}
      </div>
    );
  }
  return (
    <select className="select" aria-label={t.title} value={t.selected ?? ''} disabled={disabled} onChange={(e) => onChange(Number(e.target.value))}>
      {t.selected === null && (
        <option value="" disabled>
          Custom
        </option>
      )}
      {t.options.map((label, i) => (
        <option key={label} value={i}>
          {label}
        </option>
      ))}
    </select>
  );
}

function RangeControl({ setting: t, disabled, onCommit }: { setting: TweakState; disabled: boolean; onCommit: (value: number) => void }) {
  const range = t.range!;
  const saved = t.value ?? range.min;
  const [value, setValue] = useState(saved);
  const [shown, setShown] = useState(saved);
  if (shown !== saved) {
    setShown(saved);
    setValue(saved);
  }
  return (
    <div className="range-control">
      <Slider label={t.title} value={value} min={range.min} max={range.max} step={range.step} disabled={disabled || t.value === null} onChange={setValue} onCommit={onCommit} />
      <span className="range-value">{t.value === null ? 'n/a' : `${value}${range.unit}`}</span>
    </div>
  );
}
