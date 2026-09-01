import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useNavigate } from "@tanstack/react-router"
import { useAlertsStore } from "@/stores/alerts"
import { alertKey, marketTypeToGroupKey } from "@/types/alert"
import type { Alert } from "@/types/alert"
import { Card, CardContent } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"

function signalOf(a: Alert) {
  const diff = a.payload.cluster_mean_diff
  const p05 = a.payload.statistics.p05_diff
  const p95 = a.payload.statistics.p95_diff
  if (p05 !== null && diff < p05) return "below_p05" as const
  if (p95 !== null && diff > p95) return "above_p95" as const
  return "inside" as const
}

const SignalBadge = memo(function SignalBadge({ alert }: { alert: Alert }) {
  const s = signalOf(alert)
  if (s === "below_p05") return <Badge variant="destructive" className="text-[10px]">▼ below p05</Badge>
  if (s === "above_p95") return <Badge className="bg-emerald-600 text-white text-[10px] hover:bg-emerald-600">▲ above p95</Badge>
  return <Badge variant="outline" className="text-[10px]">inside</Badge>
})

const AlertRow = memo(function AlertRow({
  alert,
  pulsed,
  onOpen,
}: {
  alert: Alert
  pulsed: boolean
  onOpen: (a: Alert) => void
}) {
  const time = alert.timestamp.slice(11, 19)
  return (
    <TableRow
      onClick={() => onOpen(alert)}
      className={`cursor-pointer ${pulsed ? "animate-[pulse_1.2s_ease] bg-amber-50 dark:bg-amber-950/30" : ""}`}
    >
      <TableCell className="font-mono text-xs whitespace-nowrap">{time}</TableCell>
      <TableCell className="max-w-[260px] truncate font-mono text-xs" title={alert.payload.cluster_key}>
        {alert.payload.cluster_key}
      </TableCell>
      <TableCell className="whitespace-nowrap text-xs">
        <span className="font-medium">{alert.payload.market_type}</span>
        <span className="text-muted-foreground"> · {alert.payload.outcome}</span>
      </TableCell>
      <TableCell className="text-xs font-mono text-right">{alert.payload.cluster_mean_diff.toFixed(3)}</TableCell>
      <TableCell className="text-center">
        <SignalBadge alert={alert} />
      </TableCell>
      <TableCell className="font-mono text-xs text-right">{alert.payload.statistics.p05_diff?.toFixed(3) ?? "–"}</TableCell>
      <TableCell className="font-mono text-xs text-right">{alert.payload.statistics.p95_diff?.toFixed(3) ?? "–"}</TableCell>
      <TableCell className="font-mono text-xs text-right">{alert.payload.statistics.samples}</TableCell>
    </TableRow>
  )
})

function ClusterGroup({
  clusterKey,
  alerts,
  pulsedKeys,
  onOpen,
  onClearCluster,
}: {
  clusterKey: string
  alerts: Alert[]
  pulsedKeys: Set<string>
  onOpen: (a: Alert) => void
  onClearCluster: (clusterKey: string) => void
}) {
  const [open, setOpen] = useState(true)
  return (
    <Card className="overflow-hidden py-0 gap-0">
      <div className="flex w-full items-center gap-2 px-3 py-2.5 bg-muted/40">
        <button
          onClick={() => setOpen((v) => !v)}
          className="flex flex-1 items-center gap-2 text-left min-w-0"
        >
          <span className="text-xs font-mono font-semibold truncate flex-1" title={clusterKey}>
            {clusterKey}
          </span>
          <Badge variant="secondary" className="text-[10px] shrink-0">{alerts.length}</Badge>
          <span className="text-muted-foreground text-xs shrink-0">{open ? "−" : "+"}</span>
        </button>
        <Button
          variant="ghost"
          size="xs"
          onClick={(e) => {
            e.stopPropagation()
            onClearCluster(clusterKey)
          }}
          className="shrink-0 h-6 px-2 text-xs"
          aria-label={`Clear ${clusterKey}`}
        >
          Clear
        </Button>
      </div>
      {open && (
        <Table>
          <TableHeader>
            <TableRow className="hover:bg-transparent">
              <TableHead className="w-[72px] text-xs">Time</TableHead>
              <TableHead className="text-xs">Market</TableHead>
              <TableHead className="text-xs text-right">Diff</TableHead>
              <TableHead className="text-xs text-center">Signal</TableHead>
              <TableHead className="text-xs text-right">p05</TableHead>
              <TableHead className="text-xs text-right">p95</TableHead>
              <TableHead className="text-xs text-right">n</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {alerts.map((a) => (
              <AlertRow key={alertKey(a)} alert={a} pulsed={pulsedKeys.has(alertKey(a))} onOpen={onOpen} />
            ))}
          </TableBody>
        </Table>
      )}
    </Card>
  )
}

export function AlertsPage() {
  const alerts = useAlertsStore((s) => s.alerts)
  const clear = useAlertsStore((s) => s.clear)
  const clearCluster = useAlertsStore((s) => s.clearCluster)
  const navigate = useNavigate()

  const [search, setSearch] = useState("")
  const [market, setMarket] = useState("all")
  const [signal, setSignal] = useState<"all" | "below_p05" | "above_p95" | "inside">("all")
  const [grouped, setGrouped] = useState(true)
  const [paused, setPaused] = useState(false)
  const frozenRef = useRef<Alert[] | null>(null)
  const prevMapRef = useRef<Map<string, string>>(new Map())
  const [pulsedKeys, setPulsedKeys] = useState<Set<string>>(new Set())

  useEffect(() => {
    if (paused && frozenRef.current === null) frozenRef.current = alerts
    if (!paused) frozenRef.current = null
  }, [paused, alerts])

  const source = paused && frozenRef.current ? frozenRef.current : alerts
  const queuedCount = paused && frozenRef.current ? alerts.length - frozenRef.current.length : 0
  const newKeysSincePause = useMemo(() => {
    if (!paused || !frozenRef.current) return new Set<string>()
    const frozen = new Set(frozenRef.current.map((a) => alertKey(a)))
    return new Set(alerts.filter((a) => !frozen.has(alertKey(a))).map((a) => alertKey(a)))
  }, [paused, alerts])

  useEffect(() => {
    const nextPulsed = new Set<string>()
    for (const a of source) {
      const k = alertKey(a)
      const prev = prevMapRef.current.get(k)
      const cur = JSON.stringify(a.payload)
      if (prev !== undefined && prev !== cur) nextPulsed.add(k)
      prevMapRef.current.set(k, cur)
    }
    for (const k of Array.from(prevMapRef.current.keys())) {
      if (!source.some((a) => alertKey(a) === k)) prevMapRef.current.delete(k)
    }
    if (nextPulsed.size > 0) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setPulsedKeys((prev) => new Set([...prev, ...nextPulsed]))
      const t = setTimeout(() => {
        setPulsedKeys((prev) => {
          const n = new Set(prev)
          for (const k of nextPulsed) n.delete(k)
          return n
        })
      }, 1500)
      return () => clearTimeout(t)
    }
  }, [source])

  const marketOptions = useMemo(() => {
    const s = new Set(source.map((a) => a.payload.market_type))
    return ["all", ...Array.from(s).sort()]
  }, [source])

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase()
    return source.filter((a) => {
      if (q) {
        const hay = `${a.payload.cluster_key} ${a.payload.market_type} ${a.payload.outcome}`.toLowerCase()
        if (!hay.includes(q)) return false
      }
      if (market !== "all" && a.payload.market_type !== market) return false
      if (signal !== "all" && signalOf(a) !== signal) return false
      return true
    })
  }, [source, search, market, signal])

  const groupedData = useMemo(() => {
    const m = new Map<string, Alert[]>()
    for (const a of filtered) {
      const k = a.payload.cluster_key
      if (!m.has(k)) m.set(k, [])
      m.get(k)!.push(a)
    }
    const entries = Array.from(m.entries()).sort(([a], [b]) => a.localeCompare(b))
    for (const [, arr] of entries) arr.sort((x, y) => alertKey(x).localeCompare(alertKey(y)))
    return entries
  }, [filtered])

  const openAlert = useCallback(
    (a: Alert) => {
      navigate({
        to: "/",
        search: { cluster: a.payload.cluster_key, market: marketTypeToGroupKey(a.payload.market_type) } as never,
      })
    },
    [navigate],
  )

  return (
    <div className="p-4 space-y-3 max-w-6xl mx-auto">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold flex items-center gap-2">
          Alerts <Badge variant="secondary">{source.length}</Badge>
          {paused && <Badge variant="outline" className="border-amber-300 text-amber-700">paused</Badge>}
        </h1>
        <div className="ml-auto flex items-center gap-2">
          <Button variant={paused ? "secondary" : "outline"} size="sm" onClick={() => setPaused((v) => !v)}>
            {paused ? `Resume${queuedCount > 0 ? ` (+${queuedCount})` : ""}` : "Pause"}
          </Button>
          <Button variant={grouped ? "secondary" : "outline"} size="sm" onClick={() => setGrouped((v) => !v)}>
            {grouped ? "Grouped" : "Flat"}
          </Button>
          <Button variant="outline" size="sm" onClick={clear} disabled={source.length === 0}>
            Clear
          </Button>
        </div>
      </div>

      <Card className="py-3 gap-2">
        <CardContent className="flex flex-wrap gap-2 items-center">
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search cluster / market / outcome…"
            className="h-8 flex-1 min-w-[180px] rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          />
          <select
            value={market}
            onChange={(e) => setMarket(e.target.value)}
            className="h-8 rounded-md border border-input bg-background px-2 text-sm"
          >
            {marketOptions.map((m) => (
              <option key={m} value={m}>{m === "all" ? "All markets" : m}</option>
            ))}
          </select>
          <select
            value={signal}
            onChange={(e) => setSignal(e.target.value as never)}
            className="h-8 rounded-md border border-input bg-background px-2 text-sm"
          >
            <option value="all">All signals</option>
            <option value="below_p05">▼ below p05</option>
            <option value="above_p95">▲ above p95</option>
            <option value="inside">inside</option>
          </select>
          <span className="text-xs text-muted-foreground ml-auto">
            {filtered.length} / {source.length} · stable position · updates flash
          </span>
        </CardContent>
      </Card>

      {filtered.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground text-sm">
            {source.length === 0 ? "No alerts yet — waiting for SSE from /sse/alerts" : "No alerts match filters"}
          </CardContent>
        </Card>
      ) : grouped ? (
        <div className="space-y-2">
          {groupedData.map(([clusterKey, list]) => (
            <ClusterGroup key={clusterKey} clusterKey={clusterKey} alerts={list} pulsedKeys={pulsedKeys} onOpen={openAlert} onClearCluster={clearCluster} />
          ))}
          {paused && newKeysSincePause.size > 0 && (
            <Card className="border-dashed">
              <CardContent className="py-3 text-xs text-center text-muted-foreground">
                {newKeysSincePause.size} new alert(s) queued — Resume to show
              </CardContent>
            </Card>
          )}
        </div>
      ) : (
        <Card className="overflow-hidden py-0">
          <Table>
            <TableHeader>
              <TableRow className="hover:bg-transparent">
                <TableHead className="text-xs">Time</TableHead>
                <TableHead className="text-xs">Cluster</TableHead>
                <TableHead className="text-xs">Market</TableHead>
                <TableHead className="text-xs text-right">Diff</TableHead>
                <TableHead className="text-xs text-center">Signal</TableHead>
                <TableHead className="text-xs text-right">p05</TableHead>
                <TableHead className="text-xs text-right">p95</TableHead>
                <TableHead className="text-xs text-right">n</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filtered.map((a) => (
                <AlertRow
                  key={alertKey(a)}
                  alert={a}
                  pulsed={pulsedKeys.has(alertKey(a)) || newKeysSincePause.has(alertKey(a))}
                  onOpen={openAlert}
                />
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </div>
  )
}
