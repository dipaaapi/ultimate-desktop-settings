import { api } from '../lib/api';
import { usePreferences } from '../lib/theme';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { Preferences as Prefs, ThemePreference } from '../lib/types';
import { Loading, Note, Row, Section, Switch } from '../components/ui';

const THEMES: { id: ThemePreference; label: string }[] = [
  { id: 'system', label: 'Match Windows' },
  { id: 'light', label: 'Light' },
  { id: 'dark', label: 'Dark' },
];

export default function Preferences() {
  const { preferences, update } = usePreferences();
  const launch = useAsync(api.getLaunchAtStartup);
  const { busy, run } = useAction();

  if (!preferences) return <Loading rows={4} />;
  const set = (patch: Partial<Prefs>) => run('prefs', () => update({ ...preferences, ...patch }));

  return (
    <>
      <Section title="Appearance">
        <div className="tabs" role="radiogroup" aria-label="App theme">
          {THEMES.map((t) => (
            <button key={t.id} type="button" role="radio" className="tab" aria-selected={preferences.theme === t.id} aria-checked={preferences.theme === t.id} onClick={() => set({ theme: t.id })}>
              {t.label}
            </button>
          ))}
        </div>
      </Section>
      <Section title="Window">
        <div className="list">
          <Row title="Always on top" description="Keeps this window above other windows.">
            <Switch label="Always on top" checked={preferences.alwaysOnTop} onChange={(v) => set({ alwaysOnTop: v })} />
          </Row>
          <Row title="Close to the tray" description="The close button hides the window so alarms and Night Light keep working. Exit from the tray icon.">
            <Switch label="Close to the tray" checked={preferences.closeToTray} onChange={(v) => set({ closeToTray: v })} />
          </Row>
          <Row title="Launch at sign-in" description="Starts quietly in the tray when you sign in to Windows.">
            <Switch
              label="Launch at sign-in"
              checked={launch.data ?? false}
              disabled={launch.data === undefined || busy === 'launch'}
              onChange={async (v) => {
                const next = await run('launch', () => api.setLaunchAtStartup(v));
                if (next !== undefined) launch.set(next);
              }}
            />
          </Row>
        </div>
        <Note>Preferences are saved in your AppData folder and kept between updates.</Note>
      </Section>
    </>
  );
}
