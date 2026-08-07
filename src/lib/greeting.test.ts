import { describe, expect, it } from 'vitest';
import { greetingForHour, personalisedGreeting } from './greeting';

describe('time-aware greeting', () => {
  it('treats the middle of the night as evening', () => {
    expect(greetingForHour(2)).toBe('Good evening');
  });

  it('uses morning, afternoon and evening dayparts', () => {
    expect(greetingForHour(8)).toBe('Good morning');
    expect(greetingForHour(14)).toBe('Good afternoon');
    expect(greetingForHour(20)).toBe('Good evening');
  });

  it('adds the Windows display name when available', () => {
    expect(personalisedGreeting(2, 'Brian')).toBe('Good evening, Brian');
    expect(personalisedGreeting(2, '')).toBe('Good evening');
  });
});
