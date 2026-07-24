import { describe, expect, it } from 'vitest';

import { validateCatalog } from './bridge';

describe('validateCatalog', () => {
  it('rejects a command without a slash name', () => {
    expect(() => validateCatalog({ commands: [{ name: 'model' }] })).toThrow();
  });
});
