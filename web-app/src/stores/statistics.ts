import { useEffect } from "react"
import { create } from "zustand"
import { sseUrl } from "@/lib/api"
import type { StatisticsByMarketType } from "@/types/statistics"

interface StatisticsStore {
  statistics: StatisticsByMarketType | null
  isConnected: boolean
  setStatistics: (s: StatisticsByMarketType) => void
  setConnected: (c: boolean) => void
}

export const useStatisticsStore = create<StatisticsStore>((set) => ({
  statistics: null,
  isConnected: false,
  setStatistics: (statistics) => set({ statistics }),
  setConnected: (isConnected) => set({ isConnected }),
}))

let globalES: EventSource | null = null
let refCount = 0

export function useStatisticsSubscription() {
  const setStatistics = useStatisticsStore((s) => s.setStatistics)
  const setConnected = useStatisticsStore((s) => s.setConnected)

  useEffect(() => {
    if (typeof window === "undefined") return
    refCount++
    if (globalES) {
      return () => {
        refCount--
        if (refCount === 0) {
          globalES?.close()
          globalES = null
        }
      }
    }

    const url = sseUrl("/statistics")
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null
    let reconnectAttempts = 0
    const maxReconnectAttempts = 10
    const baseDelay = 1000

    function connect() {
      globalES = new EventSource(url)
      globalES.onopen = () => {
        reconnectAttempts = 0
        setConnected(true)
      }
      globalES.addEventListener("StatisticsUpdated", (event) => {
        try {
          const data: { statistics: StatisticsByMarketType } = JSON.parse(
            (event as MessageEvent).data,
          )
          setStatistics(data.statistics)
        } catch {}
      })
      globalES.onerror = () => {
        globalES?.close()
        globalES = null
        setConnected(false)
        if (reconnectAttempts >= maxReconnectAttempts) return
        const delay = baseDelay * Math.pow(2, reconnectAttempts)
        reconnectAttempts++
        reconnectTimer = setTimeout(connect, delay)
      }
    }

    connect()

    return () => {
      refCount--
      if (refCount === 0) {
        globalES?.close()
        globalES = null
        if (reconnectTimer) clearTimeout(reconnectTimer)
      }
    }
  }, [setStatistics, setConnected])
}
