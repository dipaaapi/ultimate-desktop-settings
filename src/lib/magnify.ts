// Dock-style magnification: beads near the pointer grow, with a smooth
// falloff so neighbours grow a little too.

export const MAX_SCALE = 1.85;
/** Distance (px) at which a bead stops growing. */
export const RANGE = 150;

/** Scale for a bead whose center is `distance` px from the pointer. */
export function scaleFor(distance: number, maxScale = MAX_SCALE, range = RANGE): number {
  const d = Math.abs(distance);
  if (!Number.isFinite(d) || d >= range) return 1;
  // Cosine falloff: 1 at the pointer, 0 at `range`.
  const t = (Math.cos((d / range) * Math.PI) + 1) / 2;
  return 1 + (maxScale - 1) * t;
}
