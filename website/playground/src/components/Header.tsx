interface HeaderProps {
  theme: 'dark' | 'light';
  lang: 'en' | 'zh';
  onToggleTheme: () => void;
  onToggleLang: () => void;
  t: (key: string) => string;
}

export function Header({ theme, lang, onToggleTheme, onToggleLang, t }: HeaderProps) {
  return (
    <nav className="nav">
      <div className="nav-inner">
        <a href="/rust-igraph/" className="nav-logo">
          <span className="logo-text">rust-igraph</span>
        </a>
        <div className="nav-links">
          <span className="nav-active">{t('title')}</span>
          <a href="/rust-igraph/book/">{t('guide')}</a>
          <a href="/rust-igraph/rust_igraph/">{t('apiDocs')}</a>
          <a
            href="https://github.com/Totoro-jam/rust-igraph"
            className="nav-github"
            aria-label="GitHub"
          >
            <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
            </svg>
          </a>
          <button className="lang-toggle" onClick={onToggleLang} title="Switch language">
            {lang === 'en' ? '中文' : 'EN'}
          </button>
          <button className="theme-toggle" aria-label="Toggle theme" onClick={onToggleTheme}>
            {theme === 'dark' ? (
              <svg className="icon-sun" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="5" />
                <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
              </svg>
            ) : (
              <svg className="icon-moon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" />
              </svg>
            )}
          </button>
        </div>
      </div>
    </nav>
  );
}
