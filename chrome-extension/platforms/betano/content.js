(function () {
  const SSR_PATHS = ['/sport/futebol/jogos-de-hoje/', '/sport/futebol/proximas-'];
  const LIVE_PATHS = ['/live/', '/en/live/'];
  const isSSR = SSR_PATHS.some(p => location.pathname.startsWith(p));
  const isLive = LIVE_PATHS.some(p => location.pathname.startsWith(p));
  if (!isSSR && !isLive) return;

  const POLL_MS = 5000;

  (function () {
    const s = document.createElement('script');
    s.src = chrome.runtime.getURL('platforms/betano/main-world.js');
    s.onload = () => s.remove();
    (document.head || document.documentElement).appendChild(s);
  })();

  function sendToBackground(stats) {
    chrome.runtime.sendMessage({
      type: 'PLATFORM_DATA', platform: 'betano', timestamp: Date.now(), stats,
    }).catch(() => {});
  }

  function sendLiveBlocks(blocks) {
    chrome.runtime.sendMessage({
      type: 'BETANO_LIVE_BLOCKS', platform: 'betano', timestamp: Date.now(), data: { blocks },
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
      const res = await fetch('/api/sport/futebol/jogos-de-hoje/?req=s,stnf,c,mb,mbl');
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

  function normalizeLive(json) {
    const events = json?.events ?? {};
    const markets = json?.markets ?? {};
    const selections = json?.selections ?? {};
    const leagues = json?.leagues ?? {};
    const zones = json?.zones ?? {};
    const byLeague = new Map();
    for (const ev of Object.values(events)) {
      if (ev.sportId !== 'FOOT') continue;
      const home = (ev.participants || []).find(p => p.isHome);
      const away = (ev.participants || []).find(p => !p.isHome);
      if (!home || !away) continue;
      const league = leagues[ev.leagueId] ?? {};
      const blockKey = ev.leagueId;
      if (!byLeague.has(blockKey)) byLeague.set(blockKey, { name: league.name ?? '', region: zones[ev.zoneId]?.name ?? '', events: [] });
      const ms = (ev.marketIdList || []).map(id => markets[id]).filter(Boolean).map(m => ({ ...m, id: String(m.id), selections: (m.selectionIdList || []).map(sid => selections[sid]).filter(Boolean).map(s => ({ ...s, id: String(s.id) })) }));
      byLeague.get(blockKey).events.push({ ...ev, id: String(ev.id), name: `${home.name} - ${away.name}`, leagueName: league.name ?? '', regionName: zones[ev.zoneId]?.name ?? '', markets: ms });
    }
    return [...byLeague.values()];
  }

  async function fetchLive() {
    try {
      const res = await fetch('/en/danae-webapi/api/live/overview/latest?includeVirtuals=true&queryLanguageId=1&queryOperatorId=7', { headers: { accept: 'application/json' } });
      if (!res.ok) return;
      const json = await res.json();
      const blocks = normalizeLive(json);
      if (blocks.length === 0) return;
      sendLiveBlocks(blocks);
    } catch (_) {}
  }

  if (isSSR) {
    fetchAndParse();
    setInterval(fetchAndParse, POLL_MS);
  }
  if (isLive) {
    fetchLive();
    setInterval(fetchLive, POLL_MS);
  }
})();
