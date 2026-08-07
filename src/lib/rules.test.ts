import { describe, expect, it } from 'vitest';
import type { DownloadFile, Rule, RuleConditionType } from './types';
import { evaluateRules, ruleMatches } from './rules';

const file: DownloadFile = {
  path: 'C:\\Downloads\\REPORT-Q2.pdf',
  name: 'REPORT-Q2.pdf',
  extension: 'pdf',
  size: 12_000_000,
  modifiedAt: Date.now() - 40 * 86_400_000,
  kind: 'pdf'
};

function rule(conditionType: RuleConditionType, conditionValue: string, id: string = conditionType): Rule {
  return { id, name: id, conditionType, conditionValue, actionType: 'move', enabled: true, matches: 0 };
}

describe('ruleMatches', () => {
  it.each([
    ['extension', '.PDF'],
    ['contains', 'port-q'],
    ['startsWith', 'report'],
    ['endsWith', 'Q2.PDF'],
    ['glob', 'REPORT-*.pdf'],
    ['regex', '^report-[A-Z][0-9]\\.pdf$'],
    ['size', '10'],
    ['age', '30']
  ] as const)('matches the %s condition', (type, value) => {
    expect(ruleMatches(file, rule(type, value))).toBe(true);
  });

  it('treats invalid regular expressions as a non-match', () => {
    expect(ruleMatches(file, rule('regex', '[broken'))).toBe(false);
  });

  it('uses the first enabled rule only', () => {
    const results = evaluateRules([file], [rule('extension', 'pdf', 'first'), rule('contains', 'report', 'second')]);
    expect(results).toHaveLength(1);
    expect(results[0].rule.id).toBe('first');
  });
});
