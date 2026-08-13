import { useEffect, useState } from "react"
import { sseUrl } from "@/lib/api"
import type { StatisticsByMarketType } from "@/types/statistics"

export function useClusterStatistics() {
  const [statistics, setStatistics] = useState<StatisticsByMarketType | null>(null)
  const [isConnected, setIsConnected] = useState(false)

  useEffect(() => {
    const url = sseUrl("/statistics")
    let es: EventSource | null = null
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null
    let reconnectAttempts = 0
    const maxReconnectAttempts = 10
    const baseDelay = 1000

    function connect() {
      es = new EventSource(url)

      es.onopen = () => {
        reconnectAttempts = 0
        setIsConnected(true)
      }

      es.addEventListener("StatisticsUpdated", (event) => {
        try {
          const data: { statistics: StatisticsByMarketType } = JSON.parse(
            (event as MessageEvent).data,
          )
          setStatistics(data.statistics)
        } catch {
          // skip malformed events
        }
      })

      es.onerror = () => {
        es?.close()
        setIsConnected(false)
        scheduleReconnect()
      }
    }

    function scheduleReconnect() {
      if (reconnectAttempts >= maxReconnectAttempts) return
      const delay = baseDelay * Math.pow(2, reconnectAttempts)
      reconnectAttempts++
      reconnectTimer = setTimeout(connect, delay)
    }

    connect()

    return () => {
      es?.close()
      if (reconnectTimer) clearTimeout(reconnectTimer)
    }
  }, [])

  return { statistics, isConnected }
}