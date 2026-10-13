import { describe, expect, it } from 'vitest';
import { MAX_SCALE, RANGE, scaleFor } from '../lib/magnify';
import { isVertical, spreadOffsets } from '../lib/layout';
import { formatBytes, formatDuration, formatStopwatch, timeAgo } from '../lib/format';
import { ALL_BEADS, CATEGORIES } from '../beads';

describe('scaleFor', () => {
  it('is largest under the pointer and 1 outside the range', () => {
    expect(scaleFor(0)).toBeCloseTo(MAX_SCALE);
    expect(scaleFor(RANGE)).toBe(1);
    expect(scaleFor(RANGE * 3)).toBe(1);
    expect(scaleFor(Number.NaN)).toBe(1);
  });

  it('falls off smoothly and symmetrically', () => {
    const near = scaleFor(30);
    const far = scaleFor(100);
    expect(near).toBeGreaterThan(far);
    expect(far).toBeGreaterThan(1);
    expect(scaleFor(-30)).toBeCloseTo(near);
  });
});

describe('spreadOffsets', () => {
  it('pushes neighbours away from a growing bead', () => {
    const offsets = spreadOffsets([0, 100, 200], [0, 40, 0]);
    expect(offsets).toEqual([-20, 0, 20]);
  });

  it('does nothing when nothing grows', () => {
    expect(spreadOffsets([0, 50, 100], [0, 0, 0])).toEqual([0, 0, 0]);
  });
});

describe('isVertical', () => {
  it('keeps the rod horizontal on wide windows', () => {
    expect(isVertical(1280, 720)).toBe(false);
    expect(isVertical(800, 800)).toBe(false);
  });

  it('stands the rod up on tall or narrow windows', () => {
    expect(isVertical(390, 844)).toBe(true);
    expect(isVertical(500, 400)).toBe(true);
    expect(isVertical(900, 1200)).toBe(true);
  });
});

describe('format', () => {
  it('formats bytes', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(1_382_400_000)).toBe('1.3 GB');
  });

  it('formats durations', () => {
    expect(formatDuration(65_000)).toBe('01:05');
    expect(formatDuration(3_725_000)).toBe('1:02:05');
    expect(formatStopwatch(61_230)).toBe('01:01.23');
  });

  it('describes how long ago', () => {
    const now = 10_000_000;
    expect(timeAgo(now - 10_000, now)).toBe('just now');
    expect(timeAgo(now - 5 * 60_000, now)).toBe('5 min ago');
  });
});

describe('beads', () => {
  it('have unique ids and logo colors', () => {
    const ids = ALL_BEADS.map((b) => b.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(Object.values(CATEGORIES).map((c) => c.color)).toEqual(['#5EEAD4', '#60A5FA', '#C084FC', '#FCD34D']);
  });
});
