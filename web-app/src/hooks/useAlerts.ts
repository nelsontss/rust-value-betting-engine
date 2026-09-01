import { useEffect } from "react"
import { sseUrl } from "@/lib/api"
import { useAlertsStore } from "@/stores/alerts"
import type { Alert } from "@/types/alert"

let globalES: EventSource | null = null
let refCount = 0

export function useAlertsSubscription() {
  const addAlerts = useAlertsStore((s) => s.addAlerts)

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

    const url = sseUrl("/sse/alerts")
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null
    let attempts = 0
    const maxAttempts = 10
    const baseDelay = 1000

    const pending = new Map<string, Alert>()
    let flushTimer: ReturnType<typeof setTimeout> | null = null

    function queueAlert(raw: Record<string, unknown>) {
      const payload = (raw.payload as Alert["payload"]) ?? (raw as unknown as Alert["payload"])
      if (!payload?.cluster_key || !payload?.market_type || !payload?.outcome) return
      const key = `${payload.cluster_key}_${payload.market_type}_${payload.outcome}`
      const alert: Alert = {
        id: key,
        type: (raw.type as string) ?? "MarketClusterDiffDivergency",
        timestamp: new Date().toISOString(),
        payload,
      }
      pending.set(key, alert)
      if (!flushTimer) {
        flushTimer = setTimeout(() => {
          flushTimer = null
          if (pending.size === 0) return
          const batch = Array.from(pending.values())
          pending.clear()
          addAlerts(batch)
        }, 250)
      }
    }

    function connect() {
      globalES = new EventSource(url)
      globalES.addEventListener("Alert", (event) => {
        attempts = 0
        try {
          queueAlert(JSON.parse((event as MessageEvent).data))
        } catch {}
      })
      globalES.onmessage = (event) => {
        try {
          const raw = JSON.parse((event as MessageEvent).data)
          if (!raw.payload && !raw.cluster_key) return
          queueAlert(raw)
        } catch {}
      }
      globalES.onerror = () => {
        globalES?.close()
        globalES = null
        if (attempts >= maxAttempts) return
        const delay = baseDelay * Math.pow(2, attempts)
        attempts++
        reconnectTimer = setTimeout(connect, delay)
      }
    }

    connect()
    return () => {
      refCount--
      if (flushTimer) clearTimeout(flushTimer)
      if (refCount === 0) {
        globalES?.close()
        globalES = null
        if (reconnectTimer) clearTimeout(reconnectTimer)
      }
    }
  }, [addAlerts])
}
