export function createAssetResolver(localBase) {
  const local = localBase instanceof URL ? localBase : new URL(localBase, import.meta.url);
  return (path) => {
    if (typeof path !== 'string' || !path || path.startsWith('/')) {
      throw new TypeError('Asset paths must be non-empty relative paths');
    }
    const parts = path.split('/');
    if (parts.some(part => !part || part === '.' || part === '..')) {
      throw new TypeError('Invalid asset path');
    }
    return new URL(parts.map(encodeURIComponent).join('/'), local).href;
  };
}
