(function () {
  const SSR_PATHS = ['/sport/futebol/jogos-de-hoje/', '/sport/futebol/proximas-'];
  if (!SSR_PATHS.some(p => location.pathname.startsWith(p))) return;

  const API_URL = '/api/sport/futebol/jogos-de-hoje/?req=s,stnf,c,mb,mbl';
  const POLL_MS = 5000;

  // ---- inject MAIN world listener for window.__oddsScraper ----
  (function () {
    const s = document.createElement('script');
    s.src = chrome.runtime.getURL('platforms/betano/main-world.js');
    s.onload = () => s.remove();
    (document.head || document.documentElement).appendChild(s);
  })();

  // ---- helpers ----

  function sendToBackground(stats) {
    chrome.runtime.sendMessage({
      type: 'PLATFORM_DATA', platform: 'betano', timestamp: Date.now(), stats,
    }).catch(() => {});
  }

  function exposeOnWindow(raw) {
    try {
      const summary = {
        updatedAt: Date.now(),
        leagues: (raw.blocks || []).map(b => ({ id: b.id, name: b.name, eventCount: (b.events || []).length })),
        totalEvents: (raw.blocks || []).reduce((s, b) => s + (b.events || []).length, 0),
        raw,
      };
      window.postMessage({ type: 'ODDS_SCRAPER_DATA', platform: 'betano', payload: summary }, '*');
    } catch (_) {}
  }

  function saveToStorage(raw) {
    try {
      chrome.storage.local.set({ betano_latest: { timestamp: Date.now(), data: raw } }).catch(() => {});
    } catch (_) {}
  }

  async function fetchAndParse() {
    try {
      const res = await fetch(API_URL);
      if (!res.ok) return;
      const json = await res.json();
      const raw = json?.data;
      if (!raw) return;

      const blocks = raw.blocks ?? [];
      let events = 0, markets = 0;
      for (const b of blocks) {
        events += (b.events || []).length;
        for (const e of b.events || []) markets += (e.markets || []).length;
      }
      const stats = { events, leagues: blocks.length, markets };

      exposeOnWindow(raw);
      saveToStorage(raw);
      sendToBackground(stats);
    } catch (_) {}
  }

  fetchAndParse();
  setInterval(fetchAndParse, POLL_MS);
})();
