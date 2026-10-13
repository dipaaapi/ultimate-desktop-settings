import { useState } from 'react';
import { Open20Regular, Speaker220Regular, SpeakerMute20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import type { AudioState } from '../lib/types';
import { Badge, Button, ErrorBox, Loading, Note, Row, Section, Slider } from '../components/ui';

/** A volume slider that moves freely and saves when released. */
function VolumeControl({
  label,
  level,
  muted,
  onLevel,
  onMute,
}: {
  label: string;
  level: number;
  muted: boolean;
  onLevel: (level: number) => void;
  onMute: (muted: boolean) => void;
}) {
  const [value, setValue] = useState(Math.round(level * 100));
  // Follow changes made elsewhere (e.g. the Windows volume keys) after a refresh.
  const [shown, setShown] = useState(level);
  if (shown !== level) {
    setShown(level);
    setValue(Math.round(level * 100));
  }
  return (
    <>
      <Button
        size="sm"
        variant="subtle"
        aria-label={muted ? `Unmute ${label}` : `Mute ${label}`}
        aria-pressed={muted}
        icon={muted ? <SpeakerMute20Regular /> : <Speaker220Regular />}
        onClick={() => onMute(!muted)}
      />
      <div style={{ width: 'min(240px, 50vw)' }}>
        <Slider label={`${label} volume`} value={value} onChange={setValue} onCommit={(v) => onLevel(v / 100)} />
      </div>
      <span className="mono" style={{ width: 36, textAlign: 'right' }}>
        {muted ? '—' : `${value}%`}
      </span>
    </>
  );
}

export default function Sound() {
  const audio = useAsync(api.audioState);
  const { run } = useAction();

  const apply = async (action: () => Promise<AudioState>) => {
    const next = await run('audio', action);
    if (next) audio.set(next);
  };

  if (audio.error) return <ErrorBox message={audio.error} onRetry={audio.reload} />;
  if (!audio.data) return <Loading rows={4} />;
  const a = audio.data;

  return (
    <>
      <Section title="Master volume">
        <div className="list">
          <Row title="All sounds" description={a.devices.find((d) => d.isDefault)?.name ?? 'Default output device'}>
            <VolumeControl
              label="Master"
              level={a.master}
              muted={a.muted}
              onLevel={(v) => apply(() => api.setMasterVolume(v))}
              onMute={(m) => apply(() => api.setMasterMute(m))}
            />
          </Row>
        </div>
      </Section>

      <Section title="Apps playing sound" actions={<Button size="sm" variant="subtle" onClick={audio.reload}>Refresh</Button>}>
        {a.sessions.length === 0 ? (
          <div className="empty">No apps are playing sound right now.</div>
        ) : (
          <div className="list">
            {a.sessions.map((s) => (
              <Row key={s.pid} title={s.name} description={s.isSystem ? 'Windows' : `Process ${s.pid}`}>
                <VolumeControl
                  label={s.name}
                  level={s.volume}
                  muted={s.muted}
                  onLevel={(v) => apply(() => api.setAppVolume(s.pid, v))}
                  onMute={(m) => apply(() => api.setAppMute(s.pid, m))}
                />
              </Row>
            ))}
          </div>
        )}
      </Section>

      <Section
        title="Output devices"
        actions={
          <Button size="sm" variant="subtle" icon={<Open20Regular />} onClick={() => run('s', () => api.openShortcut('sound'))}>
            Change in Windows
          </Button>
        }
      >
        <div className="list">
          {a.devices.map((d) => (
            <Row key={d.id} title={<>{d.name}{d.isDefault && <Badge tone="good">Default</Badge>}</>} />
          ))}
        </div>
        <Note>Windows only lets its own Settings switch the default device, so the button above opens it for you.</Note>
      </Section>
    </>
  );
}
