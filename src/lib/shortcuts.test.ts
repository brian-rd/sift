import { describe, expect, it } from 'vitest';
import { DEFAULT_SHORTCUTS, isBindableCode, shortcutLabel } from './shortcuts';

describe('shortcut preferences', () => {
  it('uses the intended four-direction default layout', () => {
    expect(DEFAULT_SHORTCUTS).toEqual({ keep: 'ArrowUp', trash: 'ArrowDown', undo: 'ArrowLeft', fileAway: 'ArrowRight' });
  });

  it('formats arrows, letter keys and digit keys for display', () => {
    expect(shortcutLabel('ArrowRight')).toBe('→');
    expect(shortcutLabel('KeyF')).toBe('F');
    expect(shortcutLabel('Digit7')).toBe('7');
  });

  it('reserves navigation and modifier-only keys', () => {
    expect(isBindableCode('Escape')).toBe(false);
    expect(isBindableCode('ControlLeft')).toBe(false);
    expect(isBindableCode('KeyK')).toBe(true);
  });
});
