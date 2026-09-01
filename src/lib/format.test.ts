import { describe, expect, it } from 'vitest';
import { formatWindowsPath } from './format';

describe('formatWindowsPath', () => {
  it('removes extended drive prefixes', () => {
    expect(formatWindowsPath('\\\\?\\C:\\Users\\Brian\\Downloads\\file.txt')).toBe(
      'C:\\Users\\Brian\\Downloads\\file.txt',
    );
  });

  it('restores conventional UNC paths', () => {
    expect(formatWindowsPath('\\\\?\\UNC\\server\\share\\file.txt')).toBe('\\\\server\\share\\file.txt');
  });

  it('leaves regular paths unchanged', () => {
    expect(formatWindowsPath('C:\\Downloads\\file.txt')).toBe('C:\\Downloads\\file.txt');
  });
});
