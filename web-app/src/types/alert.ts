export interface Alert {
  id: string
  type: string
  timestamp: string
  payload: MarketClusterDiffDivergencyPayload
}

export interface MarketClusterDiffDivergencyPayload {
  cluster_key: string
  cluster_mean_diff: number
  mean_divergency: number
  median_divergency: number
  market_type: string
  outcome: string
  statistics: {
    samples: number
    mean_diff: number
    median_diff: number | null
    p05_diff: number | null
    p25_diff: number | null
    p75_diff: number | null
    p95_diff: number | null
  }
}

export function alertKey(a: Alert): string {
  return `${a.payload.cluster_key}_${a.payload.market_type}_${a.payload.outcome}`
}

export function marketTypeToGroupKey(marketType: string): string {
  const sep = marketType.includes(":") ? ":" : marketType.includes("@") ? "@" : null
  if (!sep) return marketType
  const [variant, raw] = marketType.split(sep)
  const n = Number(raw)
  if (Number.isNaN(n)) return marketType
  const line = n / 100
  const lineStr = Number.isInteger(line) ? String(line) : String(line)
  return `${variant}@${lineStr}`
}
