import { useCallback, useEffect, useState } from 'react';
import { AnimatePresence, LayoutGroup, motion } from 'motion/react';
import { Alert24Regular } from '@fluentui/react-icons';
import { Stage } from './components/Stage';
import { Board } from './components/Board';
import { Button } from './components/ui';
import { isDesktop, onTimerFired } from './lib/api';
import { useVertical } from './lib/layout';
import type { Fired } from './lib/types';

export default function App() {
  const vertical = useVertical();
  const [openId, setOpenId] = useState<string | null>(null);
  const [ringing, setRinging] = useState<Fired | null>(null);
  const close = useCallback(() => setOpenId(null), []);

  useEffect(() => {
    let unsubscribe: (() => void) | undefined;
    onTimerFired(setRinging).then((u) => (unsubscribe = u));
    return () => unsubscribe?.();
  }, []);

  return (
    <div className="app">
      {!isDesktop && (
        <div className="preview-banner" role="note">
          Preview mode: sample data only. Install the Windows app to change real settings.
        </div>
      )}
      <LayoutGroup>
        <main className="scene">
          <Stage vertical={vertical} openId={openId} onOpen={setOpenId} onOpenAbout={() => setOpenId('about')} />
          <p className="hint">{vertical ? 'Tap a bead to open its board.' : 'Hover a bead to see its name. Click it to open its board.'}</p>
        </main>
        <Board openId={openId} onClose={close} />
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
