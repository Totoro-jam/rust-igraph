import { describe, it, expect } from 'vitest';
import en from './i18n/en.json';
import zh from './i18n/zh.json';

describe('i18n key parity', () => {
  const enKeys = Object.keys(en).sort();
  const zhKeys = Object.keys(zh).sort();

  it('en and zh have the same number of keys', () => {
    expect(enKeys.length).toBe(zhKeys.length);
  });

  it('every en key exists in zh', () => {
    const missing = enKeys.filter((k) => !(k in zh));
    expect(missing).toEqual([]);
  });

  it('every zh key exists in en', () => {
    const missing = zhKeys.filter((k) => !(k in en));
    expect(missing).toEqual([]);
  });

  it('no empty values in en', () => {
    const empties = enKeys.filter((k) => (en as Record<string, string>)[k]?.trim() === '');
    expect(empties).toEqual([]);
  });

  it('no empty values in zh', () => {
    const empties = zhKeys.filter((k) => (zh as Record<string, string>)[k]?.trim() === '');
    expect(empties).toEqual([]);
  });
});
