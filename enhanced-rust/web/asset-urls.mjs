export const DEFAULT_R2_PAYLOAD_BASE = new URL(
  'bo1-press-kit/payload/',
  'https://r2.houseofkublai.com/'
);

export function createAssetResolver(localBase, payloadBase = DEFAULT_R2_PAYLOAD_BASE) {
  const local = localBase instanceof URL ? localBase : new URL(localBase, import.meta.url);
  const remote = payloadBase instanceof URL ? payloadBase : new URL(payloadBase);
  return (path) => {
    if (typeof path !== 'string' || !path || path.startsWith('/')) {
      throw new TypeError('Asset paths must be non-empty relative paths');
    }
    const parts = path.split('/');
    if (parts.some(part => !part || part === '.' || part === '..')) {
      throw new TypeError('Invalid asset path');
    }
    if (parts[0] === 'payload') {
      if (parts.length < 2) throw new TypeError('Missing R2 payload key');
      return new URL(parts.slice(1).map(encodeURIComponent).join('/'), remote).href;
    }
    return new URL(parts.map(encodeURIComponent).join('/'), local).href;
  };
}
