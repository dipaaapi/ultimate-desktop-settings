import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { ArrowMinimize20Regular, Power20Regular } from '@fluentui/react-icons';

/**
 * The round title in the middle of the rod. Clicking it shows the window
 * buttons: minimize to the tray, or shut the app down completely.
 */
export function TitleBadge({ onMinimize, onShutdown }: { onMinimize: () => void; onShutdown: () => void }) {
  const [controls, setControls] = useState(false);

  useEffect(() => {
    if (!controls) return;
    const t = setTimeout(() => setControls(false), 6000);
    return () => clearTimeout(t);
  }, [controls]);

  return (
    <div className={`title-badge-wrap ${controls ? 'controls-open' : ''}`}>
      <button
        type="button"
        className="title-badge"
        onClick={() => setControls((v) => !v)}
        aria-expanded={controls}
        aria-label="Ultimate Windows Desktop Settings. Show minimize and shut down."
      >
        <span className="badge-bezel" aria-hidden="true" />
        <span className="badge-face">
          <span className="badge-kicker">Ultimate</span>
          <span className="badge-title">
            Windows
            <br />
            Desktop
          </span>
          <span className="badge-kicker">Settings</span>
          <span className="badge-version">v2</span>
        </span>
      </button>
      <AnimatePresence>
        {controls && (
          <motion.div
            className="badge-controls"
            initial={{ opacity: 0, y: 6, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 6, scale: 0.9 }}
            transition={{ duration: 0.16 }}
          >
            <button
              type="button"
              className="badge-control"
              title="Minimize to tray"
              aria-label="Minimize to tray"
              onClick={() => {
                setControls(false);
                onMinimize();
              }}
            >
              <ArrowMinimize20Regular />
            </button>
            <button type="button" className="badge-control shutdown" title="Shut down" aria-label="Shut down" onClick={onShutdown}>
              <Power20Regular />
            </button>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
