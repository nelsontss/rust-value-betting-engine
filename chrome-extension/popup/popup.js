const port = chrome.runtime.connect({ name: 'popup' });

const platformLabels = { betano: 'Betano' };

function render(state) {
  const container = document.getElementById('platforms');
  const status = document.getElementById('status');
  const debug = document.getElementById('debug');
  const platforms = state?.platforms ?? {};
  const keys = Object.keys(platforms);

  if (keys.length === 0) {
    status.textContent = 'Waiting for data...';
    container.innerHTML = '';
    debug.textContent = 'Service worker connected.';
    return;
  }

  status.textContent = `${keys.length} platform(s) active`;

  container.innerHTML = keys.map((key) => {
    const p = platforms[key];
    const label = platformLabels[key] || key;
    const stats = p.stats || {};
    const time = p.lastUpdate ? new Date(p.lastUpdate).toLocaleTimeString() : '-';

    return `
      <div class="platform-card">
        <div class="name">${label}</div>
        <div class="stats">
          <div class="stat">
            <span class="value">${stats.events ?? '?'}</span>
            <span class="label">Events</span>
          </div>
          <div class="stat">
            <span class="value">${stats.leagues ?? '?'}</span>
            <span class="label">Leagues</span>
          </div>
          <div class="stat">
            <span class="value">${stats.markets ?? '?'}</span>
            <span class="label">Markets</span>
          </div>
        </div>
        <div class="meta">Last: ${time}</div>
      </div>
    `;
  }).join('');

  const nativeStatus = state.nativeConnected ? '🟢 native' : '🔴 native';
  debug.textContent = `${nativeStatus}`;
}

port.postMessage({ type: 'GET_STATE' });

port.onMessage.addListener((msg) => {
  if (msg.type === 'STATE') render(msg.payload);
});

setInterval(() => port.postMessage({ type: 'GET_STATE' }), 2000);
