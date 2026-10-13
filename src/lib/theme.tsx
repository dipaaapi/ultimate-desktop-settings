import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from 'react';
import { api } from './api';
import type { Preferences } from './types';

interface PrefsContext {
  preferences: Preferences | undefined;
  update: (next: Preferences) => Promise<Preferences>;
}

const Context = createContext<PrefsContext>({ preferences: undefined, update: async (p) => p });

/** Loads preferences once and applies the app theme to the page. */
export function PreferencesProvider({ children }: { children: ReactNode }) {
  const [preferences, setPreferences] = useState<Preferences>();

  useEffect(() => {
    api.getPreferences().then(setPreferences).catch(() => undefined);
  }, []);

  useEffect(() => {
    const theme = preferences?.theme ?? 'system';
    const root = document.documentElement;
    if (theme === 'system') root.removeAttribute('data-theme');
    else root.setAttribute('data-theme', theme);
  }, [preferences?.theme]);

  const update = useCallback(async (next: Preferences) => {
    setPreferences(next);
    try {
      const saved = await api.setPreferences(next);
      setPreferences(saved);
      return saved;
    } catch (error) {
      // Re-read what was actually saved (e.g. Night Light switched itself off).
      setPreferences(await api.getPreferences());
      throw error;
    }
  }, []);

  return <Context.Provider value={{ preferences, update }}>{children}</Context.Provider>;
}

// eslint-disable-next-line react-refresh/only-export-components
export const usePreferences = () => useContext(Context);
