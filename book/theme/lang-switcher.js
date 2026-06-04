// Language switcher for mdBook — toggles between /book/ (EN) and /book/zh/ (ZH)
(function () {
  var toolbar = document.querySelector('.right-buttons');
  if (!toolbar) return;

  var path = window.location.pathname;
  var isZh = path.indexOf('/book/zh/') !== -1;
  var currentLang = isZh ? 'zh' : 'en';

  var btn = document.createElement('button');
  btn.className = 'icon-button';
  btn.type = 'button';
  btn.title = isZh ? 'Switch to English' : '切换到中文';
  btn.setAttribute('aria-label', btn.title);
  btn.innerHTML = isZh ? 'EN' : '中文';
  btn.style.cssText = 'font-size:14px;font-weight:600;padding:0 4px;cursor:pointer;';

  btn.addEventListener('click', function () {
    var newPath;
    if (isZh) {
      newPath = path.replace('/book/zh/', '/book/');
    } else {
      newPath = path.replace('/book/', '/book/zh/');
    }
    window.location.pathname = newPath;
  });

  toolbar.insertBefore(btn, toolbar.firstChild);
})();
