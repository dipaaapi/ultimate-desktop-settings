import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { motion, motionValue, useSpring, type MotionValue } from 'motion/react';
import { CATEGORIES, LEFT_BEADS, RIGHT_BEADS, type BeadDef } from '../beads';
import { scaleFor } from '../lib/magnify';
import { spreadOffsets } from '../lib/layout';
import { TitleBadge } from './TitleBadge';

const SPRING = { stiffness: 420, damping: 32, mass: 0.6 };

interface ItemMotion {
  scale: MotionValue<number>;
  shift: MotionValue<number>;
}

export function Stage({
  vertical,
  openId,
  onOpen,
  onOpenAbout,
}: {
  vertical: boolean;
  openId: string | null;
  onOpen: (id: string) => void;
  onOpenAbout: () => void;
}) {
  // In the upright layout the title sits on top and every bead hangs below it.
  const order: (BeadDef | 'badge')[] = useMemo(
    () => (vertical ? ['badge', ...LEFT_BEADS, ...RIGHT_BEADS] : [...LEFT_BEADS, 'badge', ...RIGHT_BEADS]),
    [vertical],
  );
  const beadCount = LEFT_BEADS.length + RIGHT_BEADS.length;
  const trackRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLElement | null)[]>([]);
  const centers = useRef<number[]>([]);
  const sizes = useRef<number[]>([]);
  const [hovered, setHovered] = useState<string | null>(null);

  // Raw targets; each item smooths them with its own spring.
  const motions = useMemo<ItemMotion[]>(
    () => order.map(() => ({ scale: motionValue(1), shift: motionValue(0) })),
    [order],
  );

  const measure = useCallback(() => {
    centers.current = itemRefs.current.map((el) => (el ? (vertical ? el.offsetTop + el.offsetHeight / 2 : el.offsetLeft + el.offsetWidth / 2) : 0));
    sizes.current = itemRefs.current.map((el) => (el ? (vertical ? el.offsetHeight : el.offsetWidth) : 0));
  }, [vertical]);

  useLayoutEffect(() => {
    measure();
    const observer = new ResizeObserver(measure);
    if (contentRef.current) observer.observe(contentRef.current);
    return () => observer.disconnect();
  }, [measure]);

  // Start with the title badge in view when the rod is longer than the window.
  useLayoutEffect(() => {
    const track = trackRef.current;
    if (!track || vertical) return;
    const badgeIndex = order.indexOf('badge');
    const center = centers.current[badgeIndex] ?? 0;
    track.scrollLeft = Math.max(0, center - track.clientWidth / 2);
  }, [order, vertical]);

  const magnifyAt = useCallback(
    (position: number | null) => {
      const extra = order.map((item, i) => {
        if (item === 'badge' || position === null) return 0;
        return (scaleFor(position - centers.current[i]) - 1) * sizes.current[i];
      });
      const shifts = spreadOffsets(centers.current, extra);
      order.forEach((item, i) => {
        const size = sizes.current[i] || 1;
        motions[i].scale.set(item === 'badge' ? 1 : 1 + extra[i] / size);
        motions[i].shift.set(shifts[i]);
      });
      if (position === null) {
        setHovered(null);
        return;
      }
      // The bead under the pointer gets its name shown.
      let best: string | null = null;
      let bestDistance = Infinity;
      order.forEach((item, i) => {
        if (item === 'badge') return;
        const distance = Math.abs(position - centers.current[i]);
        if (distance < bestDistance && distance <= sizes.current[i] * 0.75) {
          best = item.id;
          bestDistance = distance;
        }
      });
      setHovered(best);
    },
    [motions, order],
  );

  const onPointerMove = (e: React.PointerEvent) => {
    if (e.pointerType === 'touch') return;
    const rect = contentRef.current?.getBoundingClientRect();
    if (!rect) return;
    magnifyAt(vertical ? e.clientY - rect.top : e.clientX - rect.left);
  };

  // A mouse wheel scrolls the rod sideways.
  useEffect(() => {
    const track = trackRef.current;
    if (!track || vertical) return;
    const onWheel = (e: WheelEvent) => {
      if (Math.abs(e.deltaY) > Math.abs(e.deltaX) && track.scrollWidth > track.clientWidth) {
        e.preventDefault();
        track.scrollLeft += e.deltaY;
      }
    };
    track.addEventListener('wheel', onWheel, { passive: false });
    return () => track.removeEventListener('wheel', onWheel);
  }, [vertical]);

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const keys = vertical ? ['ArrowUp', 'ArrowDown'] : ['ArrowLeft', 'ArrowRight'];
    if (!keys.includes(e.key)) return;
    const buttons = Array.from(contentRef.current?.querySelectorAll<HTMLButtonElement>('[data-bead]') ?? []);
    const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (index < 0) return;
    e.preventDefault();
    const next = buttons[(index + (e.key === keys[1] ? 1 : -1) + buttons.length) % buttons.length];
    next.focus();
  };

  return (
    <div className={`stage ${vertical ? 'vertical' : 'horizontal'}`}>
      <div className="track" ref={trackRef} onPointerMove={onPointerMove} onPointerLeave={() => magnifyAt(null)}>
        <div className="track-content" ref={contentRef} onKeyDown={onKeyDown} role="toolbar" aria-orientation={vertical ? 'vertical' : 'horizontal'} aria-label={`Settings, ${beadCount} beads`}>
          <div className="rod" aria-hidden="true">
            <span className="rod-cap start" />
            <span className="rod-cap end" />
          </div>
          {order.map((item, i) =>
            item === 'badge' ? (
              <MovingItem key="badge" motion={motions[i]} innerRef={(el) => (itemRefs.current[i] = el)} vertical={vertical} className="badge-slot">
                <TitleBadge onClick={onOpenAbout} />
              </MovingItem>
            ) : (
              <MovingItem key={item.id} motion={motions[i]} innerRef={(el) => (itemRefs.current[i] = el)} vertical={vertical} className="bead-slot">
                <Bead
                  bead={item}
                  open={openId === item.id}
                  showLabel={hovered === item.id}
                  onOpen={() => onOpen(item.id)}
                  onFocus={() => magnifyAt(centers.current[i])}
                  onBlur={() => magnifyAt(null)}
                />
              </MovingItem>
            ),
          )}
        </div>
      </div>
    </div>
  );
}

function MovingItem({
  motion: m,
  innerRef,
  vertical,
  className,
  children,
}: {
  motion: ItemMotion;
  innerRef: (el: HTMLElement | null) => void;
  vertical: boolean;
  className: string;
  children: React.ReactNode;
}) {
  const scale = useSpring(m.scale, SPRING);
  const shift = useSpring(m.shift, SPRING);
  return (
    <motion.div ref={innerRef} className={className} style={vertical ? { y: shift } : { x: shift }}>
      <motion.div className="slot-scale" style={{ scale }}>
        {children}
      </motion.div>
    </motion.div>
  );
}

function Bead({
  bead,
  open,
  showLabel,
  onOpen,
  onFocus,
  onBlur,
}: {
  bead: BeadDef;
  open: boolean;
  showLabel: boolean;
  onOpen: () => void;
  onFocus: () => void;
  onBlur: () => void;
}) {
  const Icon = bead.icon;
  const color = CATEGORIES[bead.category].color;
  return (
    <div className="bead-wrap">
      <button
        type="button"
        data-bead={bead.id}
        className="bead"
        aria-label={`${bead.title}: ${bead.subtitle}`}
        aria-expanded={open}
        onClick={onOpen}
        onFocus={onFocus}
        onBlur={onBlur}
        style={{ '--bead': color } as React.CSSProperties}
      >
        {open ? (
          <span className="bead-socket" />
        ) : (
          <motion.span className="bead-sphere" layoutId={`bead-${bead.id}`} style={{ borderRadius: 999 }}>
            <Icon className="bead-icon" />
          </motion.span>
        )}
      </button>
      <span className={`bead-label ${showLabel && !open ? 'visible' : ''}`} aria-hidden="true">
        {bead.title}
      </span>
    </div>
  );
}
