export function readStoredJson<T>(key: string, fallback: T, isValid: (value: unknown) => value is T): T {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(key) ?? 'null');
    return isValid(value) ? value : fallback;
  } catch {
    return fallback;
  }
}

export function writeStoredJson(key: string, value: unknown) {
  localStorage.setItem(key, JSON.stringify(value));
}
