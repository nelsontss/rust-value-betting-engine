import { create } from "zustand"
import type { Alert } from "@/types/alert"
import { alertKey } from "@/types/alert"

interface AlertsState {
  alerts: Alert[]
  toasts: Alert[]
  addAlert: (alert: Alert) => void
  addAlerts: (alerts: Alert[]) => void
  dismissToast: (id: string) => void
  clear: () => void
}

export const useAlertsStore = create<AlertsState>((set) => ({
  alerts: [],
  toasts: [],
  addAlert: (alert) =>
    set((s) => {
      const key = alertKey(alert)
      const existingIdx = s.alerts.findIndex((a) => alertKey(a) === key)
      let nextAlerts: Alert[]
      if (existingIdx >= 0) {
        nextAlerts = [alert, ...s.alerts.filter((_, i) => i !== existingIdx)].slice(0, 200)
      } else {
        nextAlerts = [alert, ...s.alerts].slice(0, 200)
      }
      const toastIdx = s.toasts.findIndex((a) => alertKey(a) === key)
      let nextToasts: Alert[]
      if (toastIdx >= 0) {
        nextToasts = [alert, ...s.toasts.filter((_, i) => i !== toastIdx)].slice(0, 5)
      } else {
        nextToasts = [alert, ...s.toasts].slice(0, 5)
      }
      return { alerts: nextAlerts, toasts: nextToasts }
    }),
  addAlerts: (batch) =>
    set((s) => {
      if (batch.length === 0) return s
      const map = new Map<string, Alert>()
      for (const a of s.alerts) map.set(alertKey(a), a)
      for (const a of batch) map.delete(alertKey(a))
      const nextAlerts = [...batch.slice().reverse(), ...Array.from(map.values())].slice(0, 200)
      const toastMap = new Map<string, Alert>()
      for (const t of s.toasts) toastMap.set(alertKey(t), t)
      for (const a of batch) toastMap.delete(alertKey(a))
      const nextToasts = [...batch.slice().reverse(), ...Array.from(toastMap.values())].slice(0, 5)
      return { alerts: nextAlerts, toasts: nextToasts }
    }),
  dismissToast: (id) =>
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
  clear: () => set({ alerts: [], toasts: [] }),
}))
