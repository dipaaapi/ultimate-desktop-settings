import { Suspense, useEffect, useRef } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { Dismiss24Regular } from '@fluentui/react-icons';
import { CATEGORIES, findBead } from '../beads';
import { Loading } from './ui';

export function Board({ openId, onClose }: { openId: string | null; onClose: () => void }) {
  const bead = openId ? findBead(openId) : undefined;
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!bead) return;
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose();
    window.addEventListener('keydown', onKey);
    const t = setTimeout(() => closeRef.current?.focus(), 50);
    return () => {
      window.removeEventListener('keydown', onKey);
      clearTimeout(t);
    };
  }, [bead, onClose]);

  return (
    <AnimatePresence>
      {bead && (
        <div className="board-layer" key="layer">
          <motion.div
            className="board-backdrop"
            onClick={onClose}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          />
          <motion.div
            key={bead.id}
            layoutId={`bead-${bead.id}`}
            className="board"
            role="dialog"
            aria-modal="true"
            aria-labelledby="board-title"
            style={{ borderRadius: 28, '--bead': CATEGORIES[bead.category].color } as React.CSSProperties}
            transition={{ type: 'spring', stiffness: 260, damping: 30 }}
            exit={{ opacity: 0, transition: { duration: 0.18 } }}
          >
            <motion.div
              className="board-inner"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1, transition: { delay: 0.12, duration: 0.2 } }}
              exit={{ opacity: 0, transition: { duration: 0.08 } }}
            >
              <header className="board-head">
                <span className="board-orb" aria-hidden="true">
                  <bead.icon />
                </span>
                <div className="board-heading">
                  <h2 id="board-title">{bead.title}</h2>
                  <p>{bead.subtitle}</p>
                </div>
                <button ref={closeRef} type="button" className="icon-btn" onClick={onClose} aria-label="Close">
                  <Dismiss24Regular />
                </button>
              </header>
              <div className="board-body">
                <Suspense fallback={<Loading rows={4} />}>
                  <bead.Board />
                </Suspense>
              </div>
            </motion.div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
