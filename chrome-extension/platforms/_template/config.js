// Template for adding a new platform.
// 1. Copy this directory: cp -r platforms/_template platforms/mybookie
// 2. Edit config.js with your platform details
// 3. Edit content.js with your extraction logic
// 4. Register match patterns in manifest.json content_scripts
// 5. Add the label to popup/popup.js platformLabels

const PLATFORM_CONFIG = {
  name: 'mybookie',
  label: 'MyBookie',

  matchPatterns: ['https://www.mybookie.com/*'],

  pollIntervalMs: 30000,

  endpoints: {
    // API endpoints for this platform
  },

  marketTypeNames: {
    // Market type code to display name mapping
  },

  parseData(data) {
    // Return { leagues, events, markets }
    return { leagues: 0, events: 0, markets: 0 };
  },
};
