import type { DownloadFile, Rule } from './types';

export interface RuleMatch {
  file: DownloadFile;
  rule: Rule;
}

function globToRegExp(pattern: string): RegExp {
  const escaped = pattern
    .replace(/[.+^${}()|[\]\\]/g, '\\$&')
    .replace(/\*/g, '.*')
    .replace(/\?/g, '.');
  return new RegExp(`^${escaped}$`, 'i');
}

export function ruleMatches(file: DownloadFile, rule: Rule, now = Date.now()): boolean {
  const name = file.name.toLowerCase();
  const value = rule.conditionValue.trim();
  const valueLower = value.toLowerCase();

  switch (rule.conditionType) {
    case 'extension':
      return file.extension.toLowerCase() === valueLower.replace(/^\./, '');
    case 'contains':
      return name.includes(valueLower);
    case 'startsWith':
      return name.startsWith(valueLower);
    case 'endsWith':
      return name.endsWith(valueLower);
    case 'glob':
      try { return globToRegExp(value).test(file.name); } catch { return false; }
    case 'regex':
      try { return new RegExp(value, 'i').test(file.name); } catch { return false; }
    case 'size': {
      const megabytes = Number(value);
      return Number.isFinite(megabytes) && file.size > megabytes * 1_000_000;
    }
    case 'age': {
      const days = Number(value);
      return Number.isFinite(days) && now - file.modifiedAt > days * 86_400_000;
    }
  }
}

export function evaluateRules(files: DownloadFile[], rules: Rule[]): RuleMatch[] {
  const active = rules.filter((rule) => rule.enabled);
  return files.flatMap((file) => {
    const rule = active.find((candidate) => ruleMatches(file, candidate));
    return rule ? [{ file, rule }] : [];
  });
}
