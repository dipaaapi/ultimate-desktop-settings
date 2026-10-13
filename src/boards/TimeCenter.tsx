import { useEffect, useMemo, useState } from 'react';
import { Add20Regular, Delete20Regular, Flag20Regular, Pause20Regular, Play20Regular, Speaker220Regular, Stop20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { formatDuration, formatStopwatch, pad2 } from '../lib/format';
import { usePreferences } from '../lib/theme';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { Alarm, Tone } from '../lib/types';
import { Badge, Button, ConfirmButton, ErrorBox, Loading, Note, Row, Section, Switch, TextInput } from '../components/ui';

type Tab = 'timer' | 'alarms' | 'stopwatch' | 'world';
const TABS: { id: Tab; label: string }[] = [
  { id: 'timer', label: 'Timer' },
  { id: 'alarms', label: 'Alarms' },
  { id: 'stopwatch', label: 'Stopwatch' },
  { id: 'world', label: 'World clock' },
];

/** Re-renders every `ms` while `active` is true and returns the current time. */
function useNow(ms: number, active = true) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!active) return;
    const id = setInterval(() => setNow(Date.now()), ms);
    return () => clearInterval(id);
  }, [ms, active]);
  return now;
}

export default function TimeCenter() {
  const [tab, setTab] = useState<Tab>('timer');
  return (
    <>
      <div className="tabs" role="tablist" aria-label="Time Center">
        {TABS.map((t) => (
          <button key={t.id} type="button" role="tab" className="tab" aria-selected={tab === t.id} onClick={() => setTab(t.id)}>
            {t.label}
          </button>
        ))}
      </div>
      {tab === 'timer' && <Timer />}
      {tab === 'alarms' && <Alarms />}
      {tab === 'stopwatch' && <Stopwatch />}
      {tab === 'world' && <WorldClock />}
    </>
  );
}

function Timer() {
  const countdown = useAsync(api.countdownState);
  const { run } = useAction();
  const [minutes, setMinutes] = useState(5);
  const [seconds, setSeconds] = useState(0);
  const [label, setLabel] = useState('');
  const now = useNow(250, !!countdown.data);
  const remaining = countdown.data ? countdown.data.endsAtMs - now : 0;

  useEffect(() => {
    if (countdown.data && remaining <= 0) countdown.set(null);
  }, [countdown, remaining]);

  if (countdown.error) return <ErrorBox message={countdown.error} onRetry={countdown.reload} />;
  if (countdown.loading && countdown.data === undefined) return <Loading rows={2} />;

  if (countdown.data) {
    return (
      <div className="card big-clock-card">
        <div className="big-clock">{formatDuration(remaining)}</div>
        <div className="row-desc">{countdown.data.label}</div>
        <Button
          variant="danger"
          icon={<Stop20Regular />}
          onClick={async () => {
            await run('cancel', api.cancelCountdown);
            countdown.set(null);
          }}
        >
          Cancel timer
        </Button>
      </div>
    );
  }

  const total = minutes * 60 + seconds;
  return (
    <>
      <div className="grid presets">
        {[1, 5, 10, 15, 25, 45].map((m) => (
          <button key={m} type="button" className="tile" onClick={() => (setMinutes(m), setSeconds(0))}>
            <span className="tile-title">{m} min</span>
          </button>
        ))}
      </div>
      <form
        className="card form-row"
        style={{ marginTop: 14 }}
        onSubmit={async (e) => {
          e.preventDefault();
          const started = await run('start', () => api.startCountdown(total, label.trim() || 'Timer'));
          if (started) countdown.set(started);
        }}
      >
        <TextInput label="Minutes" type="number" min={0} max={1440} value={minutes} onChange={(e) => setMinutes(Math.max(0, Number(e.target.value) || 0))} />
        <TextInput label="Seconds" type="number" min={0} max={59} value={seconds} onChange={(e) => setSeconds(Math.min(59, Math.max(0, Number(e.target.value) || 0)))} />
        <TextInput label="Label" placeholder="Timer" value={label} onChange={(e) => setLabel(e.target.value)} />
        <button type="submit" className="btn btn-primary" disabled={total <= 0}>
          <span className="btn-icon"><Play20Regular /></span>
          Start
        </button>
      </form>
      <Note>The timer keeps running in the tray when the window is closed, and rings even if this board is not open.</Note>
    </>
  );
}

const DAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const TONES: Tone[] = ['default', 'information', 'warning', 'critical'];

function describeDays(days: number[]) {
  if (days.length === 0) return 'Once';
  if (days.length === 7) return 'Every day';
  if (days.length === 5 && [1, 2, 3, 4, 5].every((d) => days.includes(d))) return 'Weekdays';
  return [...days].sort().map((d) => DAYS[d]).join(', ');
}

function Alarms() {
  const alarms = useAsync(api.listAlarms);
  const { run } = useAction();
  const [time, setTime] = useState('07:00');
  const [label, setLabel] = useState('');
  const [days, setDays] = useState<number[]>([]);
  const [tone, setTone] = useState<Tone>('default');

  const save = async (alarm: Alarm) => {
    const next = await run(alarm.id, () => api.saveAlarm(alarm));
    if (next) alarms.set(next);
    return next;
  };

  if (alarms.error) return <ErrorBox message={alarms.error} onRetry={alarms.reload} />;
  if (!alarms.data) return <Loading rows={3} />;

  return (
    <>
      <Section title="Your alarms">
        {alarms.data.length === 0 ? (
          <div className="empty">No alarms yet.</div>
        ) : (
          <div className="list">
            {alarms.data.map((a) => (
              <Row key={a.id} title={<><span className="mono" style={{ fontSize: 16 }}>{pad2(a.hour)}:{pad2(a.minute)}</span> {a.label}</>} description={describeDays(a.days)}>
                <ConfirmButton
                  size="sm"
                  variant="subtle"
                  aria-label={`Delete ${a.label}`}
                  icon={<Delete20Regular />}
                  confirmLabel="Delete?"
                  onConfirm={async () => {
                    const next = await run(a.id, () => api.deleteAlarm(a.id));
                    if (next) alarms.set(next);
                  }}
                />
                <Switch label={`${a.label} on`} checked={a.enabled} onChange={(v) => save({ ...a, enabled: v })} />
              </Row>
            ))}
          </div>
        )}
      </Section>
      <Section title="New alarm">
        <form
          className="card"
          onSubmit={async (e) => {
            e.preventDefault();
            const [hour, minute] = time.split(':').map(Number);
            const next = await save({ id: `alarm-${Date.now().toString(36)}`, label: label.trim() || 'Alarm', hour, minute, days, enabled: true, tone });
            if (next) {
              setLabel('');
              setDays([]);
            }
          }}
        >
          <div className="form-row">
            <TextInput label="Time" type="time" required value={time} onChange={(e) => setTime(e.target.value)} />
            <TextInput label="Label" placeholder="Alarm" value={label} onChange={(e) => setLabel(e.target.value)} />
            <label className="field">
              <span className="field-label">Sound</span>
              <select className="select" value={tone} onChange={(e) => setTone(e.target.value as Tone)}>
                {TONES.map((t) => (
                  <option key={t} value={t}>
                    {t[0].toUpperCase() + t.slice(1)}
                  </option>
                ))}
              </select>
            </label>
            <Button aria-label="Play sound" icon={<Speaker220Regular />} onClick={() => run('tone', () => api.testTone(tone))} />
          </div>
          <div className="day-picker" role="group" aria-label="Repeat on">
            {DAYS.map((d, i) => (
              <button
                key={d}
                type="button"
                className="tab"
                aria-pressed={days.includes(i)}
                aria-selected={days.includes(i)}
                onClick={() => setDays(days.includes(i) ? days.filter((x) => x !== i) : [...days, i])}
              >
                {d}
              </button>
            ))}
            <span className="row-desc">{describeDays(days)}</span>
          </div>
          <button type="submit" className="btn btn-primary">
            <span className="btn-icon"><Add20Regular /></span>
            Add alarm
          </button>
        </form>
      </Section>
    </>
  );
}

function Stopwatch() {
  const [elapsed, setElapsed] = useState(0);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [laps, setLaps] = useState<number[]>([]);
  const now = useNow(31, startedAt !== null);
  const current = elapsed + (startedAt !== null ? now - startedAt : 0);

  return (
    <>
      <div className="card big-clock-card">
        <div className="big-clock">{formatStopwatch(current)}</div>
        <div className="section-actions">
          {startedAt === null ? (
            <Button variant="primary" icon={<Play20Regular />} onClick={() => setStartedAt(Date.now())}>
              {current ? 'Resume' : 'Start'}
            </Button>
          ) : (
            <Button icon={<Pause20Regular />} onClick={() => (setElapsed(current), setStartedAt(null))}>
              Pause
            </Button>
          )}
          <Button icon={<Flag20Regular />} disabled={startedAt === null} onClick={() => setLaps([current, ...laps])}>
            Lap
          </Button>
          <Button variant="subtle" disabled={!current} onClick={() => (setElapsed(0), setStartedAt(null), setLaps([]))}>
            Reset
          </Button>
        </div>
      </div>
      {laps.length > 0 && (
        <div className="list" style={{ marginTop: 14 }}>
          {laps.map((t, i) => (
            <Row key={laps.length - i} title={`Lap ${laps.length - i}`} description={`+${formatStopwatch(t - (laps[i + 1] ?? 0))}`}>
              <span className="mono">{formatStopwatch(t)}</span>
            </Row>
          ))}
        </div>
      )}
    </>
  );
}

function zones(): string[] {
  try {
    return Intl.supportedValuesOf('timeZone');
  } catch {
    return ['UTC', 'Asia/Manila', 'Asia/Tokyo', 'Europe/London', 'America/New_York', 'America/Los_Angeles', 'Australia/Sydney'];
  }
}

function WorldClock() {
  const { preferences, update } = usePreferences();
  const { run } = useAction();
  const now = useNow(1000);
  const all = useMemo(() => zones(), []);
  const [pick, setPick] = useState('Asia/Manila');

  if (!preferences) return <Loading rows={3} />;
  const clocks = preferences.worldClocks;
  const local = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const fmt = (zone: string, opts: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat(undefined, { timeZone: zone, ...opts }).format(now);
  const setClocks = (worldClocks: string[]) => run('wc', () => update({ ...preferences, worldClocks }));

  return (
    <>
      <div className="list">
        {[local, ...clocks].map((zone, i) => (
          <Row
            key={`${zone}-${i}`}
            title={<>{zone.split('/').pop()?.replace(/_/g, ' ')}{i === 0 && <Badge tone="accent">This PC</Badge>}</>}
            description={fmt(zone, { weekday: 'long', month: 'short', day: 'numeric', timeZoneName: 'short' })}
          >
            <span className="mono" style={{ fontSize: 18 }}>{fmt(zone, { hour: '2-digit', minute: '2-digit' })}</span>
            {i > 0 && (
              <Button size="sm" variant="subtle" aria-label={`Remove ${zone}`} icon={<Delete20Regular />} onClick={() => setClocks(clocks.filter((_, j) => j !== i - 1))} />
            )}
          </Row>
        ))}
      </div>
      <form
        className="card form-row"
        style={{ marginTop: 14 }}
        onSubmit={(e) => {
          e.preventDefault();
          if (!clocks.includes(pick)) setClocks([...clocks, pick]);
        }}
      >
        <label className="field">
          <span className="field-label">Add a city</span>
          <select className="select" value={pick} onChange={(e) => setPick(e.target.value)}>
            {all.map((z) => (
              <option key={z} value={z}>
                {z.replace(/_/g, ' ')}
              </option>
            ))}
          </select>
        </label>
        <button type="submit" className="btn btn-primary" disabled={clocks.includes(pick) || clocks.length >= 12}>
          <span className="btn-icon"><Add20Regular /></span>
          Add
        </button>
      </form>
    </>
  );
}
