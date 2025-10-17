// Global setup for API_URL (environment imports removed)
declare global {
  interface Window {
    API_URL: string;
  }
  var window: Window & typeof globalThis;
}

// Set up API_URL for testing
if (typeof window !== 'undefined') {
  (window as any).API_URL = 'http://localhost:3000/api';
} else {
  // Node.js environment - create minimal window mock
  (globalThis as any).window = { API_URL: 'http://localhost:3000/api' };
}

export {};
