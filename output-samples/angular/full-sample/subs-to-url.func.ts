// Generated utility function for URL building
// Do not modify this file manually

import { environment } from '@env/environment';

export function subsToUrl(
  url: string,
  params?: { [key: string]: string | number | boolean | null | undefined },
  queryParams?: { [key: string]: string | number | boolean | null | undefined }
): string {
  if (params) {
    for (const key in params) {
      if (params.hasOwnProperty(key)) {
        const regex = new RegExp(':' + key + '($|/)');
        url = url.replace(regex, params[key] + '$1');
      }
    }
  }
  
  if (queryParams) {
    const qs = Object.keys(queryParams)
      .filter((key) => queryParams[key] !== null && queryParams[key] !== undefined)
      .map((key) => {
        const value = encodeURIComponent(queryParams[key]!);
        return `${key}=${value}`;
      })
      .join('&');
      
    if (qs.length > 0) {
      url += '?' + qs;
    }
  }

  const injectedConfig = (window as any).API_CONFIG;
  if (injectedConfig) {
    return injectedConfig.BACKEND_API_URL + url;
  }

  return environment.apiUrl + url;
}
