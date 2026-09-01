import { memo, useCallback } from "react"
import { useNavigate } from "@tanstack/react-router"
import { useAlertsStore } from "@/stores/alerts"
import { alertKey, marketTypeToGroupKey } from "@/types/alert"
import type { Alert } from "@/types/alert"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

const AlertCard = memo(function AlertCard({ alert, onOpen }: { alert: Alert; onOpen: (a: Alert) => void }) {
  const time = alert.timestamp.slice(11, 19)
  return (
    <Card
      className="cursor-pointer hover:bg-accent/50 transition-colors"
      style={{ contentVisibility: "auto", containIntrinsicSize: "0 120px" } as React.CSSProperties}
      onClick={() => onOpen(alert)}
    >
      <CardHeader className="pb-2">
        <CardTitle className="text-sm flex items-center gap-2 flex-wrap">
          <span>{alert.payload.market_type}</span>
          <Badge variant="outline" className="text-xs">{alert.payload.outcome}</Badge>
          <span className="text-xs font-normal text-muted-foreground ml-auto">{time}</span>
        </CardTitle>
      </CardHeader>
      <CardContent className="text-xs space-y-1">
        <p className="font-mono truncate text-muted-foreground">{alert.payload.cluster_key}</p>
        <div className="flex gap-3 flex-wrap items-center">
          <span>diff <b>{alert.payload.cluster_mean_diff.toFixed(3)}</b></span>
          <span className={alert.payload.cluster_mean_diff < (alert.payload.statistics.p05_diff ?? -Infinity) ? "text-red-600 font-bold" : alert.payload.cluster_mean_diff > (alert.payload.statistics.p95_diff ?? Infinity) ? "text-emerald-600 font-bold" : ""}>
            {alert.payload.cluster_mean_diff < (alert.payload.statistics.p05_diff ?? -Infinity) ? "▼ abaixo p05" : alert.payload.cluster_mean_diff > (alert.payload.statistics.p95_diff ?? Infinity) ? "▲ acima p95" : ""}
          </span>
          <span>p05 <b>{alert.payload.statistics.p05_diff?.toFixed(3) ?? "–"}</b></span>
          <span>p95 <b>{alert.payload.statistics.p95_diff?.toFixed(3) ?? "–"}</b></span>
          <span>samples {alert.payload.statistics.samples}</span>
        </div>
        <div className="flex gap-3 flex-wrap text-muted-foreground">
          <span>mean {alert.payload.statistics.mean_diff.toFixed(3)}</span>
          <span>median {alert.payload.statistics.median_diff?.toFixed(3) ?? "–"}</span>
        </div>
      </CardContent>
    </Card>
  )
})

export function AlertsPage() {
  const alerts = useAlertsStore((s) => s.alerts)
  const clear = useAlertsStore((s) => s.clear)
  const navigate = useNavigate()

  const openAlert = useCallback(
    (a: Alert) => {
      navigate({
        to: "/",
        search: {
          cluster: a.payload.cluster_key,
          market: marketTypeToGroupKey(a.payload.market_type),
        } as never,
      })
    },
    [navigate],
  )

  return (
    <div className="p-4 space-y-4 max-w-3xl mx-auto">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">
          Alerts <Badge variant="secondary" className="ml-2">{alerts.length}</Badge>
        </h1>
        {alerts.length > 0 && (
          <Button variant="outline" size="sm" onClick={clear}>
            Clear
          </Button>
        )}
      </div>

      {alerts.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground text-sm">
            No alerts yet — waiting for SSE events from /sse/alerts
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2">
          {alerts.map((a) => (
            <AlertCard key={alertKey(a)} alert={a} onOpen={openAlert} />
          ))}
        </div>
      )}
    </div>
  )
}
