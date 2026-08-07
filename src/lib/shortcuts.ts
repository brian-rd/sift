import type { ShortcutBindings } from './types';

export const DEFAULT_SHORTCUTS: ShortcutBindings = {
  keep: 'ArrowUp',
  trash: 'ArrowDown',
  undo: 'ArrowLeft',
  fileAway: 'ArrowRight'
};

const labels: Record<string, string> = {
  ArrowUp: '↑',
  ArrowDown: '↓',
  ArrowLeft: '←',
  ArrowRight: '→',
  Space: 'Space',
  Enter: 'Enter',
  Backspace: 'Backspace',
  Delete: 'Delete'
};

export function shortcutLabel(code: string) {
  return labels[code] ?? code.replace(/^Key/, '').replace(/^Digit/, '');
}

export function isBindableCode(code: string) {
  return !['Escape', 'Tab', 'CapsLock', 'ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(code);
}
