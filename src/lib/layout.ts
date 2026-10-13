import { useEffect, useState } from 'react';

/** Tall or very narrow windows stand the rod upright. */
export function isVertical(width: number, height: number): boolean {
  return width < 520 || height > width * 1.1;
}

export function useVertical(): boolean {
  const read = () => (typeof window === 'undefined' ? false : isVertical(window.innerWidth, window.innerHeight));
  const [vertical, setVertical] = useState(read);
  useEffect(() => {
    const onResize = () => setVertical(read());
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);
  return vertical;
}

/**
 * How far each item slides so growing beads push their neighbours apart
 * instead of overlapping them. `extra[j]` is how much item j grew (px).
 */
export function spreadOffsets(centers: number[], extra: number[]): number[] {
  return centers.map((ci, i) =>
    centers.reduce((sum, cj, j) => (j === i || cj === ci ? sum : sum + Math.sign(ci - cj) * (extra[j] / 2)), 0),
  );
}
