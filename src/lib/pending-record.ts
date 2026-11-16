// Reading evidence must distinguish an absent action from unreadable delivery state.
export type PendingRecord<T> =
  | { pending: T | null; error?: undefined }
  | { pending: null; error: string };

export function readPendingRecord<T>(
  key: string,
  decode: (value: unknown) => T | null,
): PendingRecord<T> {
  try {
    const stored = localStorage.getItem(key);
    if (stored === null) return { pending: null };
    const value: unknown = JSON.parse(stored);
    const pending = decode(value);
    if (pending) return { pending };
  } catch {
    // Preserve the record and fence submission even when storage itself is unavailable.
  }
  return {
    pending: null,
    error:
      "The desktop could not read pending action evidence. Use fresh private development state: stop owned work, deliberately clear the app's pending record, then reopen. The saved record has been preserved. The earlier action's outcome is unknown; no new action will be submitted here.",
  };
}
