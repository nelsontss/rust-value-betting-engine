export const MSG_TYPES = {
  PLATFORM_DATA: 'PLATFORM_DATA',
  PLATFORM_ERROR: 'PLATFORM_ERROR',
};

export function platformDataMsg(platform, source, stats) {
  return {
    type: MSG_TYPES.PLATFORM_DATA,
    platform,
    source,
    timestamp: Date.now(),
    stats,
  };
}

export function platformErrorMsg(platform, source, error) {
  return {
    type: MSG_TYPES.PLATFORM_ERROR,
    platform,
    source,
    timestamp: Date.now(),
    error,
  };
}
