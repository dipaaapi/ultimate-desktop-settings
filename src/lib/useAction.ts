import { useCallback, useState } from 'react';
import { useToast } from '../components/Toasts';
import { errorMessage } from './format';

/**
 * Runs a command from a button, shows a toast when it fails and tracks
 * which action is busy so its button can be disabled.
 */
export function useAction() {
  const toast = useToast();
  const [busy, setBusy] = useState<string>();
  const run = useCallback(
    async <T,>(key: string, action: () => Promise<T>, success?: string | ((value: T) => string)): Promise<T | undefined> => {
      setBusy(key);
      try {
        const value = await action();
        if (success) toast(typeof success === 'string' ? success : success(value), 'success');
        return value;
      } catch (error) {
        toast(errorMessage(error), 'error');
        return undefined;
      } finally {
        setBusy(undefined);
      }
    },
    [toast],
  );
  return { busy, run };
}
