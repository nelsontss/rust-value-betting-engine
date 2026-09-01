import { useEffect } from "react"
import { sseUrl } from "@/lib/api"
import { useAlertsStore } from "@/stores/alerts"
import type { Alert, ConvergencyEvent } from "@/types/alert"

let globalES: EventSource | null = null
let refCount = 0

export function useAlertsSubscription() {
  const addAlerts = useAlertsStore((s) => s.addAlerts)
  const handleConvergencyBatch = useAlertsStore((s) => s.handleConvergencyBatch)

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
    const pendingConvergency = new Map<string, ConvergencyEvent>()
    let flushTimer: ReturnType<typeof setTimeout> | null = null

    function scheduleFlush() {
      if (flushTimer) return
      flushTimer = setTimeout(() => {
        flushTimer = null
        if (pending.size > 0) {
          const batch = Array.from(pending.values())
          pending.clear()
          addAlerts(batch)
        }
        if (pendingConvergency.size > 0) {
          const batch = Array.from(pendingConvergency.values())
          pendingConvergency.clear()
          handleConvergencyBatch(batch)
        }
      }, 250)
    }

    function queueAlert(raw: Record<string, unknown>) {
      const type = (raw.type as string) ?? "MarketClusterDiffDivergency"
      const payload = (raw.payload as Record<string, unknown>) ?? raw
      const cluster_key = payload.cluster_key as string | undefined
      const market_type = payload.market_type as string | undefined
      const outcome = payload.outcome as string | undefined
      if (!cluster_key || !market_type || !outcome) return
      const key = `${cluster_key}_${market_type}_${outcome}`
      if (type === "AlertConvergency") {
        const event: ConvergencyEvent = {
          id: key,
          type: "AlertConvergency",
          timestamp: new Date().toISOString(),
          payload: payload as unknown as ConvergencyEvent["payload"],
        }
        pendingConvergency.set(key, event)
        scheduleFlush()
        return
      }
      const alert: Alert = {
        id: key,
        type,
        timestamp: new Date().toISOString(),
        payload: payload as unknown as Alert["payload"],
      }
      pending.set(key, alert)
      scheduleFlush()
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
  }, [addAlerts, handleConvergencyBatch])
}
