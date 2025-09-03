let jwt = localStorage.getItem("jwt");
if (!jwt) {
    localStorage.setItem("redirect", window.location.href);
    window.location.href = "/login";
} else {
    fetch(window.location.href, {
        headers: {
            "Authorization": `Bearer ${jwt}`
        }
    }).then(res => res.text()).then(html => {
        const parser = new DOMParser();
        const doc = parser.parseFromString(html, 'text/html');

        document.body.outerHTML = doc.body.outerHTML;
        document.head.innerHTML += doc.head.innerHTML;
    });
}

const THEME_KEY = 'site-theme';

console.log("docs main.js loaded");

function applyTheme(theme) {
  if (theme === 'dark') {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
 }

function initTheme() {
  const saved = localStorage.getItem(THEME_KEY);
  if (saved) {
    applyTheme(saved);
  } else {
    const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    applyTheme(prefersDark ? 'dark' : 'light');
  }

  setTimeout(() => {
    const btn = document.getElementById('theme-toggle');
    if (btn) {
      btn.addEventListener('click', () => {
        const isDark = document.documentElement.classList.toggle('dark');
        const newTheme = isDark ? 'dark' : 'light';
      localStorage.setItem(THEME_KEY, newTheme);
    });
  }}, 1000);
}

function initDropdowns() {
  document.querySelectorAll('.dropdown').forEach(drop => {
    const btn = drop.querySelector('.dropbtn');
    const menu = drop.querySelector('.dropdown-content');
    if (!btn || !menu) return;
    btn.addEventListener('click', (e) => {
      const expanded = btn.getAttribute('aria-expanded') === 'true';
      btn.setAttribute('aria-expanded', String(!expanded));
      menu.style.display = expanded ? 'none' : 'block';
    });
    document.addEventListener('click', (e) => {
      if (!drop.contains(e.target)) {
        menu.style.display = 'none';
        btn.setAttribute('aria-expanded', 'false');
      }
    });
  });
}

document.addEventListener('DOMContentLoaded', () => {
  initTheme();
  setTimeout(() => {
    initDropdowns();
  }, 1000);
});
