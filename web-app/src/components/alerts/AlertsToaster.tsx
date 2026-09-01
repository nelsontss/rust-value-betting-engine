import { useEffect } from "react"
import { useNavigate } from "@tanstack/react-router"
import { useAlertsStore } from "@/stores/alerts"
import { marketTypeToGroupKey } from "@/types/alert"

export function AlertsToaster() {
  const toasts = useAlertsStore((s) => s.toasts)
  const dismiss = useAlertsStore((s) => s.dismissToast)
  const navigate = useNavigate()

  useEffect(() => {
    if (toasts.length === 0) return
    const timers = toasts.map((t) =>
      setTimeout(() => dismiss(t.id), 6000)
    )
    return () => timers.forEach(clearTimeout)
  }, [toasts, dismiss])

  if (toasts.length === 0) return null

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 w-[360px] pointer-events-none">
      {toasts.map((alert) => (
        <div
          key={alert.id}
          onClick={() => {
            dismiss(alert.id)
            navigate({
              to: "/",
              search: {
                cluster: alert.payload.cluster_key,
                market: marketTypeToGroupKey(alert.payload.market_type),
              } as never,
            })
          }}
          className="pointer-events-auto cursor-pointer rounded-lg border bg-card p-3 shadow-lg hover:bg-accent transition-colors"
        >
          <div className="flex items-start justify-between gap-2">
            <p className="text-sm font-medium leading-none">
              {alert.payload.market_type} · {alert.payload.outcome}
            </p>
            <button
              onClick={(e) => { e.stopPropagation(); dismiss(alert.id) }}
              className="text-muted-foreground hover:text-foreground text-xs"
            >
              ✕
            </button>
          </div>
          <p className="text-xs text-muted-foreground mt-1 truncate">
            {alert.payload.cluster_key}
          </p>
          <p className="text-xs mt-1">
            diff {alert.payload.cluster_mean_diff.toFixed(4)} · p05 {alert.payload.statistics.p05_diff?.toFixed(3) ?? "–"} · p95 {alert.payload.statistics.p95_diff?.toFixed(3) ?? "–"} {alert.payload.cluster_mean_diff < (alert.payload.statistics.p05_diff ?? -Infinity) ? "▼" : alert.payload.cluster_mean_diff > (alert.payload.statistics.p95_diff ?? Infinity) ? "▲" : ""}
          </p>
        </div>
      ))}
    </div>
  )
}
