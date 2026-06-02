const PLATFORMS = [
  {
    name: 'betano', label: 'Betano',
    todayUrl: 'https://www.betano.pt/en/api/sport/futebol/jogos-de-hoje/?req=s,stnf,c,mb,mbl',
    upcomingUrl: 'https://www.betano.pt/en/api/upcomingcoupon/?sid=FOOT&req=la,s,stnf,c,mb,mbl',
    referer: 'https://www.betano.pt/en/sport/futebol/jogos-de-hoje/',
    pollMs: 5000,
  },
];

const platformState = {};
let nativePort = null;

function ts() { return new Date().toISOString().slice(11, 23); }

function log(...args) { console.log(`[${ts()}] [OddsScraper]`, ...args); }
function warn(...args) { console.warn(`[${ts()}] [OddsScraper]`, ...args); }

// ---- Native messaging ----

function connectNative() {
  if (nativePort) return;
  try {
    nativePort = chrome.runtime.connectNative('com.odds_scrapper.bridge');
    nativePort.onDisconnect.addListener(() => {
      log('native port disconnected');
      nativePort = null;
    });
    nativePort.onMessage.addListener((msg) => {
      log('native reply: ' + JSON.stringify(msg).slice(0, 200));
    });
    log('native port connected');
  } catch (err) {
    warn('native port failed:', err.message);
  }
}

function sendToNative(type, platform, data) {
  if (!nativePort) connectNative();
  if (!nativePort) return;
  try {
    nativePort.postMessage({ type, platform, timestamp: Date.now(), data });
  } catch (err) {
    warn('native send failed:', err.message);
    nativePort = null;
  }
}

// ---- Fetch helpers ----

async function fetchJson(url, referer) {
  const res = await fetch(url, {
    headers: { 'Referer': referer },
    cache: 'no-cache',
  });
  if (!res.ok) return null;
  return res.json();
}

function countStats(blocks) {
  let events = 0, markets = 0;
  for (const b of blocks) {
    events += (b.events || []).length;
    for (const e of b.events || []) markets += (e.markets || []).length;
  }
  return { events, leagues: blocks.length, markets };
}

// ---- Platform polling ----

async function pollToday(pf) {
  try {
    const t0 = Date.now();
    const json = await fetchJson(pf.todayUrl, pf.referer);
    const blocks = json?.data?.blocks ?? [];
    if (blocks.length === 0) return;

    const elapsed = Date.now() - t0;
    const stats = countStats(blocks);
    log(`${pf.name}/today: ${stats.events} events, ${stats.leagues} leagues, ${stats.markets} markets (${elapsed}ms)`);
    updateState(pf.name, stats);
    sendToNative('odds_update', pf.name, { blocks });
  } catch (err) {
    warn(`${pf.name}/today failed: ${err.message}`);
  }
}

async function pollUpcoming(pf) {
  try {
    const t0 = Date.now();
    const json = await fetchJson(pf.upcomingUrl, pf.referer);
    const subNavItems = json?.data?.coupons?.[0]?.subNavItems ?? [];

    const blocks = [];
    for (const day of subNavItems) {
      if ((day.events || []).length > 0) {
        blocks.push({ name: day.name, events: day.events });
      }
    }
    if (blocks.length === 0) return;

    const elapsed = Date.now() - t0;
    const stats = countStats(blocks);
    log(`${pf.name}/upcoming: ${stats.events} events, ${stats.leagues} leagues, ${stats.markets} markets (${elapsed}ms)`);
    updateState(pf.name, stats);
    sendToNative('odds_update', pf.name, { blocks });
  } catch (err) {
    warn(`${pf.name}/upcoming failed: ${err.message}`);
  }
}

function updateState(platform, stats) {
  if (!platformState[platform]) platformState[platform] = { stats };
  else platformState[platform].stats = stats;
  platformState[platform].lastUpdate = Date.now();
}

function startPolling(pf) {
  log(`starting ${pf.name} polling every ${pf.pollMs}ms`);
  pollToday(pf);
  pollUpcoming(pf);
  setInterval(() => pollToday(pf), pf.pollMs);
  setInterval(() => pollUpcoming(pf), pf.pollMs);
}

// ---- Lifecycle ----

chrome.runtime.onInstalled.addListener(() => {
  log('onInstalled');
  for (const pf of PLATFORMS) startPolling(pf);
});
chrome.runtime.onStartup.addListener(() => {
  log('onStartup');
  for (const pf of PLATFORMS) startPolling(pf);
});

// ---- Popup ----

chrome.runtime.onConnect.addListener((port) => {
  if (port.name === 'popup') {
    port.postMessage({ type: 'STATE', payload: { platforms: platformState, nativeConnected: !!nativePort } });
    port.onMessage.addListener((msg) => {
      if (msg.type === 'GET_STATE') {
        port.postMessage({ type: 'STATE', payload: { platforms: platformState, nativeConnected: !!nativePort } });
      }
    });
  }
});

// ---- Keepalive ----

chrome.alarms.create('keepalive', { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener(() => {});
