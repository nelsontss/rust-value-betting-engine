# Odds Scraper — Chrome Extension

Chrome extension that polls bookmaker APIs and forwards odds data to the Rust engine via native messaging.

## Architecture

```
background.js (service worker)
  │  setInterval → fetch bookmaker API → parse → sendToNative
  ▼
native messaging → rust-value-betting-engine bridge → ClusterService
```

The service worker runs independently without needing a visible tab. It fetches API endpoints directly every 5 seconds and sends the raw JSON response through the native messaging host to the Rust bridge binary.

## Platforms

| Platform | API Endpoint |
|----------|-------------|
| Betano | `/api/sport/futebol/jogos-de-hoje/` (today) |
| Betano | `/api/upcomingcoupon/` (upcoming days) |

## Development

1. Load unpacked extension at `chrome://extensions`
2. Native messaging host must be registered (see `native-host/`)
3. Service worker logs are visible via "Service Worker" link on the extension card

> **Note:** This extension module was AI-generated as part of the rust-value-betting-engine project. It was reviewed and adjusted for correctness but should be validated against real bookmaker API behavior before production use.
