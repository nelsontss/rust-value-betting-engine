const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:3005"

export function apiUrl(path: string): string {
  return `${API_BASE}${path}`
}

export function sseUrl(path: string): string {
  return `${API_BASE}${path}`
}
