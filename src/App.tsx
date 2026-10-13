import { useCallback, useEffect, useRef, useState } from 'react';
import { AnimatePresence, LayoutGroup, motion } from 'motion/react';
import { Alert24Regular } from '@fluentui/react-icons';
import { Stage } from './components/Stage';
import { Board } from './components/Board';
import { Button } from './components/ui';
import { useToast } from './components/Toasts';
import { api, isDesktop, onTimerFired } from './lib/api';
import { useVertical } from './lib/layout';
import type { Fired, Placement } from './lib/types';

export default function App() {
  const narrow = useVertical();
  // The desktop rod is a fixed floating strip; only the browser preview stands it upright.
  const vertical = !isDesktop && narrow;
  const toast = useToast();
  const [openId, setOpenId] = useState<string | null>(null);
  const [placement, setPlacement] = useState<Placement>('below');
  const [ringing, setRinging] = useState<Fired | null>(null);
  const openRef = useRef<string | null>(null);

  const open = useCallback(async (id: string) => {
    openRef.current = id;
    if (isDesktop) {
      // Grow the window towards the roomier side of the screen first.
      setPlacement(await api.setWindowMode('board').catch(() => 'below' as const));
    }
    if (openRef.current === id) setOpenId(id);
  }, []);
  const close = useCallback(() => {
    openRef.current = null;
    setOpenId(null);
  }, []);
  // Shrink back to the rod once the board has animated away.
  const onBoardClosed = useCallback(() => {
    if (isDesktop && openRef.current === null) void api.setWindowMode('rod');
  }, []);

  // Bead names follow the same side as boards.
  const refreshPlacement = useCallback(() => {
    if (isDesktop && openRef.current === null) api.windowPlacement().then(setPlacement, () => undefined);
  }, []);

  useEffect(() => {
    let unsubscribe: (() => void) | undefined;
    onTimerFired(setRinging).then((u) => (unsubscribe = u));
    return () => unsubscribe?.();
  }, []);

  const desktopOnly = (action: () => Promise<void>) => () => {
    if (isDesktop) void action();
    else toast('This works in the Windows app. The preview keeps running.');
  };

  return (
    <div className={`app placement-${placement} ${openId ? 'board-open' : ''}`}>
      {!isDesktop && (
        <div className="preview-banner" role="note">
          Preview mode: sample data only. Install the Windows app to change real settings.
        </div>
      )}
      <LayoutGroup>
        <main className="scene" onPointerEnter={refreshPlacement}>
          <Stage
            vertical={vertical}
            openId={openId}
            onOpen={open}
            onMinimize={desktopOnly(api.hideToTray)}
            onShutdown={desktopOnly(api.quitApp)}
          />
          {!isDesktop && (
            <p className="hint">
              {vertical ? 'Tap a bead to open its board.' : 'Hover a bead to see its name. Click it to open its board. Click the title for minimize and shut down.'}
            </p>
          )}
        </main>
        <Board openId={openId} onClose={close} onClosed={onBoardClosed} />
      </LayoutGroup>
      <AnimatePresence>
        {ringing && (
          <div className="board-layer alarm-layer">
            <motion.div className="board-backdrop" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} />
            <motion.div
              className="alarm-card"
              role="alertdialog"
              aria-labelledby="alarm-title"
              initial={{ scale: 0.85, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
            >
              <Alert24Regular className="alarm-icon" />
              <h2 id="alarm-title">{ringing.kind === 'alarm' ? 'Alarm' : "Time's up"}</h2>
              <p>{ringing.label || (ringing.kind === 'alarm' ? 'Your alarm is ringing.' : 'Your countdown timer has finished.')}</p>
              <Button variant="primary" onClick={() => setRinging(null)} autoFocus>
                Dismiss
              </Button>
            </motion.div>
          </div>
        )}
      </AnimatePresence>
    </div>
  );
}
