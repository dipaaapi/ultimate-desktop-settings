import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { errorMessage } from './format';

export interface AsyncState<T> {
  data: T | undefined;
  error: string | undefined;
  loading: boolean;
  reload: () => void;
  /** Replaces the data with a command's result (most commands return the new state). */
  set: (value: T) => void;
}

interface Snapshot<T> {
  data?: T;
  error?: string;
  loading: boolean;
}

/** Loads data for a board and keeps it fresh after actions. */
export function useAsync<T>(load: () => Promise<T>, deps: unknown[] = []): AsyncState<T> {
  const [state, setState] = useState<Snapshot<T>>({ loading: true });
  const [tick, setTick] = useState(0);
  const loadRef = useRef(load);
  useLayoutEffect(() => {
    loadRef.current = load;
  });

  useEffect(() => {
    let active = true;
    loadRef
      .current()
      .then((data) => active && setState({ data, loading: false }))
      .catch((e: unknown) => active && setState((s) => ({ ...s, error: errorMessage(e), loading: false })));
    return () => {
      active = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tick, ...deps]);

  const reload = useCallback(() => {
    setState((s) => ({ ...s, loading: true }));
    setTick((t) => t + 1);
  }, []);
  const set = useCallback((data: T) => setState({ data, loading: false }), []);
  return { data: state.data, error: state.error, loading: state.loading, reload, set };
}
