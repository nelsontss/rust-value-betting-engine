import { create } from "zustand"
import type { Alert, ConvergencyEvent } from "@/types/alert"
import { alertKey } from "@/types/alert"

interface AlertsState {
  alerts: Alert[]
  toasts: Alert[]
  convergencyToasts: ConvergencyEvent[]
  addAlert: (alert: Alert) => void
  addAlerts: (alerts: Alert[]) => void
  handleConvergency: (event: ConvergencyEvent) => void
  handleConvergencyBatch: (events: ConvergencyEvent[]) => void
  dismissToast: (id: string) => void
  dismissConvergency: (id: string) => void
  clear: () => void
  clearCluster: (clusterKey: string) => void
  removeAlert: (key: string) => void
}

export const useAlertsStore = create<AlertsState>((set) => ({
  alerts: [],
  toasts: [],
  convergencyToasts: [],
  addAlert: (alert) =>
    set((s) => {
      const key = alertKey(alert)
      const existingIdx = s.alerts.findIndex((a) => alertKey(a) === key)
      let nextAlerts: Alert[]
      if (existingIdx >= 0) {
        nextAlerts = s.alerts.slice()
        nextAlerts[existingIdx] = alert
      } else {
        nextAlerts = [alert, ...s.alerts].slice(0, 200)
      }
      const toastIdx = s.toasts.findIndex((a) => alertKey(a) === key)
      let nextToasts: Alert[]
      if (toastIdx >= 0) {
        nextToasts = s.toasts.slice()
        nextToasts[toastIdx] = alert
      } else {
        nextToasts = [alert, ...s.toasts].slice(0, 5)
      }
      return { alerts: nextAlerts, toasts: nextToasts }
    }),
  addAlerts: (batch) =>
    set((s) => {
      if (batch.length === 0) return s
      const existing = new Map<string, number>()
      s.alerts.forEach((a, i) => existing.set(alertKey(a), i))
      const nextAlerts = s.alerts.slice()
      const newOnes: Alert[] = []
      for (const a of batch) {
        const k = alertKey(a)
        const idx = existing.get(k)
        if (idx !== undefined) {
          nextAlerts[idx] = a
        } else {
          if (!newOnes.some((x) => alertKey(x) === k)) newOnes.push(a)
          existing.set(k, -1)
        }
      }
      const merged = [...newOnes.reverse(), ...nextAlerts].slice(0, 200)
      const toastMap = new Map<string, Alert>()
      for (const t of s.toasts) toastMap.set(alertKey(t), t)
      for (const a of batch) toastMap.set(alertKey(a), a)
      const nextToasts = Array.from(toastMap.values()).slice(-5).reverse()
      return { alerts: merged, toasts: nextToasts }
    }),
  handleConvergency: (event) =>
    set((s) => {
      const key = `${event.payload.cluster_key}_${event.payload.market_type}_${event.payload.outcome}`
      return {
        alerts: s.alerts.filter((a) => alertKey(a) !== key),
        toasts: s.toasts.filter((a) => alertKey(a) !== key),
        convergencyToasts: [event, ...s.convergencyToasts.filter((c) => c.id !== event.id)].slice(0, 5),
      }
    }),
  handleConvergencyBatch: (events) =>
    set((s) => {
      if (events.length === 0) return s
      const keys = new Set(events.map((e) => `${e.payload.cluster_key}_${e.payload.market_type}_${e.payload.outcome}`))
      const ids = new Set(events.map((e) => e.id))
      return {
        alerts: s.alerts.filter((a) => !keys.has(alertKey(a))),
        toasts: s.toasts.filter((a) => !keys.has(alertKey(a))),
        convergencyToasts: [...events.slice().reverse(), ...s.convergencyToasts.filter((c) => !ids.has(c.id))].slice(0, 5),
      }
    }),
  dismissToast: (id) =>
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
  dismissConvergency: (id) =>
    set((s) => ({ convergencyToasts: s.convergencyToasts.filter((c) => c.id !== id) })),
  clear: () => set({ alerts: [], toasts: [], convergencyToasts: [] }),
  clearCluster: (clusterKey) =>
    set((s) => ({
      alerts: s.alerts.filter((a) => a.payload.cluster_key !== clusterKey),
      toasts: s.toasts.filter((a) => a.payload.cluster_key !== clusterKey),
    })),
  removeAlert: (key) =>
    set((s) => ({
      alerts: s.alerts.filter((a) => alertKey(a) !== key),
      toasts: s.toasts.filter((a) => alertKey(a) !== key),
    })),
}))
