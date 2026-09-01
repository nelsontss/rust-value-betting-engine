import { useStatisticsStore, useStatisticsSubscription } from "@/stores/statistics"

export function useClusterStatistics() {
  const statistics = useStatisticsStore((s) => s.statistics)
  const isConnected = useStatisticsStore((s) => s.isConnected)

  useStatisticsSubscription()

  return { statistics, isConnected }
}