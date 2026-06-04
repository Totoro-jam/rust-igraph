import { useState, useCallback, useMemo } from 'react';
import en from '../i18n/en.json';
import zh from '../i18n/zh.json';

type Lang = 'en' | 'zh';

const messages: Record<Lang, Record<string, string>> = { en, zh };

function getInitialLang(): Lang {
  const saved = localStorage.getItem('lang');
  if (saved === 'en' || saved === 'zh') return saved;
  if (navigator.language.startsWith('zh')) return 'zh';
  return 'en';
}

export function useI18n() {
  const [lang, setLangState] = useState<Lang>(getInitialLang);

  const setLang = useCallback((l: Lang) => {
    setLangState(l);
    localStorage.setItem('lang', l);
  }, []);

  const toggleLang = useCallback(() => {
    setLang(lang === 'en' ? 'zh' : 'en');
  }, [lang, setLang]);

  const t = useMemo(() => {
    const dict = messages[lang];
    return (key: string): string => dict[key] ?? key;
  }, [lang]);

  return { lang, setLang, toggleLang, t } as const;
}
