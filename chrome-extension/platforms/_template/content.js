// Template content script for a new platform.
// Copy this to platforms/mybookie/content.js and adapt.

(function () {
  const PLATFORM = 'mybookie';

  // ---- Config ----
  const ENDPOINT_URL = '/api/...';
  const POLL_INTERVAL_MS = 30000;

  // ---- Helpers ----

  function parseData(json) {
    // Extract stats from platform-specific response shape
    return {
      leagues: 0,
      events: 0,
      markets: 0,
      raw: json,
    };
  }

  function sendStats(source, parsed) {
    chrome.runtime.sendMessage({
      type: 'PLATFORM_DATA',
      platform: PLATFORM,
      source,
      timestamp: Date.now(),
      stats: {
        leagues: parsed.leagues,
        events: parsed.events,
        markets: parsed.markets,
      },
    });
  }

  function sendError(source, err) {
    chrome.runtime.sendMessage({
      type: 'PLATFORM_ERROR',
      platform: PLATFORM,
      source,
      timestamp: Date.now(),
      error: err.message ?? String(err),
    });
  }

  async function pollEndpoint() {
    try {
      const res = await fetch(ENDPOINT_URL);
      if (!res.ok) { sendError('poll', new Error(`HTTP ${res.status}`)); return; }
      const json = await res.json();
      const parsed = parseData(json);
      sendStats('poll', parsed);
    } catch (err) {
      sendError('poll', err);
    }
  }

  // ---- Init ----

  // Platform-specific initialization
  // For SSR data: inject a script to read window.__INITIAL_STATE__
  // For REST API: just start polling

  pollEndpoint();
  setInterval(pollEndpoint, POLL_INTERVAL_MS);
})();
