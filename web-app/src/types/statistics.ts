export interface StatisticsValues {
  samples: number
  mean_diff: number
  median_diff: number | null
  p05_diff: number | null
  p25_diff: number | null
  p75_diff: number | null
  p95_diff: number | null
}

export type OutcomeStatistics = Record<string, StatisticsValues>

export type StatisticsByMarketType = Record<string, OutcomeStatistics>

export interface StatisticsUpdatedResponse {
  statistics: StatisticsByMarketType
}