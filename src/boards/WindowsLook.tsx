import { Image20Regular } from '@fluentui/react-icons';
import { api, pickPicture } from '../lib/api';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import { SettingsList } from '../components/SettingsList';
import { Button, Row, Section } from '../components/ui';

export default function WindowsLook() {
  const wallpaper = useAsync(api.getWallpaper);
  const { busy, run } = useAction();
  return (
    <SettingsList
      page="look"
      before={
        <Section title="Background">
          <div className="list">
            <Row title="Desktop picture" description={<span className="mono">{wallpaper.data ?? (wallpaper.loading ? 'Loading…' : 'Solid color or slideshow')}</span>}>
              <Button
                icon={<Image20Regular />}
                disabled={busy === 'wallpaper'}
                onClick={async () => {
                  const picked = await pickPicture();
                  if (!picked) return;
                  const next = await run('wallpaper', () => api.setWallpaper(picked), 'Wallpaper changed.');
                  if (next !== undefined) wallpaper.set(next);
                }}
              >
                Choose picture
              </Button>
            </Row>
          </div>
        </Section>
      }
    />
  );
}
