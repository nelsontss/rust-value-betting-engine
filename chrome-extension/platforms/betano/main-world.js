(function () {
  const KEY = '__oddsScraper';

  window.addEventListener('message', function (event) {
    if (event.source !== window) return;
    if (event.data?.type !== 'ODDS_SCRAPER_DATA') return;
    window[KEY] = window[KEY] || {};
    window[KEY][event.data.platform] = event.data.payload;
  });

  window[KEY] = window[KEY] || {};
})();
