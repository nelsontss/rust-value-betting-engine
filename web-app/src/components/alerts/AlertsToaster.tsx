import { useEffect } from "react"
import { useNavigate } from "@tanstack/react-router"
import { useAlertsStore } from "@/stores/alerts"
import { marketTypeToGroupKey } from "@/types/alert"

export function AlertsToaster() {
  const toasts = useAlertsStore((s) => s.toasts)
  const convergencyToasts = useAlertsStore((s) => s.convergencyToasts)
  const dismiss = useAlertsStore((s) => s.dismissToast)
  const dismissConvergency = useAlertsStore((s) => s.dismissConvergency)
  const navigate = useNavigate()

  useEffect(() => {
    if (toasts.length === 0) return
    const timers = toasts.map((t) =>
      setTimeout(() => dismiss(t.id), 6000)
    )
    return () => timers.forEach(clearTimeout)
  }, [toasts, dismiss])

  useEffect(() => {
    if (convergencyToasts.length === 0) return
    const timers = convergencyToasts.map((t) =>
      setTimeout(() => dismissConvergency(t.id), 6000)
    )
    return () => timers.forEach(clearTimeout)
  }, [convergencyToasts, dismissConvergency])

  if (toasts.length === 0 && convergencyToasts.length === 0) return null

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 w-[360px] pointer-events-none">
      {convergencyToasts.map((ev) => (
        <div
          key={`conv-${ev.id}`}
          onClick={() => {
            dismissConvergency(ev.id)
            navigate({
              to: "/",
              search: {
                cluster: ev.payload.cluster_key,
                market: marketTypeToGroupKey(ev.payload.market_type),
              } as never,
            })
          }}
          className="pointer-events-auto cursor-pointer rounded-lg border border-emerald-200 bg-emerald-50 dark:bg-emerald-950/40 dark:border-emerald-900 p-3 shadow-lg hover:bg-emerald-100 dark:hover:bg-emerald-900/30 transition-colors"
        >
          <div className="flex items-start justify-between gap-2">
            <p className="text-sm font-medium leading-none text-emerald-700 dark:text-emerald-300">
              ✓ Converged · {ev.payload.market_type} · {ev.payload.outcome}
            </p>
            <button
              onClick={(e) => { e.stopPropagation(); dismissConvergency(ev.id) }}
              className="text-emerald-600 hover:text-emerald-800 dark:text-emerald-400 text-xs"
            >
              ✕
            </button>
          </div>
          <p className="text-xs text-muted-foreground mt-1 truncate">
            {ev.payload.cluster_key}
          </p>
          <p className="text-xs mt-1 text-emerald-700 dark:text-emerald-300">
            diff {ev.payload.cluster_mean_diff.toFixed(4)} · {(ev.payload.initial_polymarket_impl_prob * 100).toFixed(1)}% → {(ev.payload.current_polymarket_impl_prob * 100).toFixed(1)}%
          </p>
        </div>
      ))}
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
