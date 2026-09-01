import { useClusterStatistics } from "@/hooks/useClusterStatistics"
import type { Cluster } from "@/types/cluster"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { cn } from "@/lib/utils"

function fmt(v: number | null | undefined, d = 4): string {
  if (v === null || v === undefined) return "–"
  return v.toFixed(d)
}


export function LiveDiffComparisonTable({
  cluster,
  filterMarket,
}: {
  cluster: Cluster
  filterMarket?: { type: string; line?: number } | null
}) {
  const { statistics } = useClusterStatistics()
  const live = cluster.live_diffs ?? {}

  function keyMatches(mt: string): boolean {
    if (!filterMarket) return false
    if (filterMarket.type === mt) return true
    const [mtType, mtLineRaw] = mt.split(/[:@]/)
    if (mtType !== filterMarket.type) return false
    if (filterMarket.line === undefined) return mtLineRaw === undefined
    if (mtLineRaw === undefined) return false
    const mtLine = mtLineRaw.includes(".") ? parseFloat(mtLineRaw) : parseInt(mtLineRaw, 10) / 100
    return Math.abs(mtLine - filterMarket.line) < 1e-9
  }

  const rows: Array<{
    marketType: string
    outcome: string
    liveDiff: number
    mean: number | null
    median: number | null
    p05: number | null
    p95: number | null
  }> = []

  for (const [mt, outcomes] of Object.entries(live)) {
    if (filterMarket && !keyMatches(mt)) continue
    for (const [out, diff] of Object.entries(outcomes)) {
      const stat = statistics?.[mt]?.[out]
      const mean = stat?.mean_diff ?? null
      const median = stat?.median_diff ?? null
      const p05 = stat?.p05_diff ?? null
      const p95 = stat?.p95_diff ?? null
      rows.push({ marketType: mt, outcome: out, liveDiff: diff, mean, median, p05, p95 })
    }
  }

  const ORDER: Record<string, number> = { MatchResult: 0, Moneyline: 1, DoubleChance: 2, Total: 3, Handicap: 4, AsianHandicap: 5 }
  rows.sort((a, b) => (ORDER[a.marketType.split(":")[0].split("@")[0]] ?? 99) - (ORDER[b.marketType.split(":")[0].split("@")[0]] ?? 99) || a.marketType.localeCompare(b.marketType) || a.outcome.localeCompare(b.outcome))

  if (!filterMarket) {
    return (
      <Card>
        <CardHeader><CardTitle className="text-sm">Live vs Historical</CardTitle></CardHeader>
        <CardContent className="text-sm text-muted-foreground">Select a market to compare live vs historical diffs</CardContent>
      </Card>
    )
  }

  if (rows.length === 0) {
    const hasPoly = cluster.games.some((g) =>
      g.platform === "Polymarket" && g.markets.some((m) => m.type === filterMarket.type && (filterMarket.line === undefined || (m as { line?: number }).line === filterMarket.line)),
    )
    const hasBookmaker = cluster.games.some((g) =>
      g.platform !== "Polymarket" && g.markets.some((m) => m.type === filterMarket.type && (filterMarket.line === undefined || (m as { line?: number }).line === filterMarket.line)),
    )
    let msg = "No live diffs for selected market"
    if (!hasPoly && !hasBookmaker) msg = "No market data for selected type"
    else if (!hasPoly) msg = "No Polymarket data — diff requires Polymarket vs bookmaker median"
    else if (!hasBookmaker) msg = "Only Polymarket — no bookmaker to compare for diff"
    return (
      <Card>
        <CardHeader><CardTitle className="text-sm">Live vs Historical</CardTitle></CardHeader>
        <CardContent className="text-sm text-amber-600 bg-amber-50 rounded p-3 m-3">{msg}</CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Live vs Historical diffs</CardTitle></CardHeader>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Market</TableHead>
              <TableHead>Outcome</TableHead>
              <TableHead className="text-right">Live</TableHead>
              <TableHead className="text-right">Hist mean</TableHead>
              <TableHead className="text-right">Hist median</TableHead>
              <TableHead className="text-right">p05</TableHead>
              <TableHead className="text-right">p95</TableHead>
              <TableHead className="text-right">Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((r) => {
              const below = r.p05 !== null && r.liveDiff < r.p05
              const above = r.p95 !== null && r.liveDiff > r.p95
              const isAlert = below || above
              return (
                <TableRow key={`${r.marketType}:${r.outcome}`} className={cn(isAlert && (below ? "bg-red-50" : "bg-emerald-50"))}>
                  <TableCell className="font-mono text-xs">{r.marketType}</TableCell>
                  <TableCell className="text-xs">{r.outcome}</TableCell>
                  <TableCell className={cn("text-right font-mono tabular-nums text-xs font-medium", isAlert && (below ? "text-red-600" : "text-emerald-600"))}>{fmt(r.liveDiff)}</TableCell>
                  <TableCell className="text-right font-mono tabular-nums text-xs">{fmt(r.mean)}</TableCell>
                  <TableCell className="text-right font-mono tabular-nums text-xs">{fmt(r.median)}</TableCell>
                  <TableCell className="text-right font-mono tabular-nums text-xs">{fmt(r.p05)}</TableCell>
                  <TableCell className="text-right font-mono tabular-nums text-xs">{fmt(r.p95)}</TableCell>
                  <TableCell className={cn("text-right text-xs font-semibold", below && "text-red-600", above && "text-emerald-600")}>{below ? "▼ abaixo p05" : above ? "▲ acima p95" : "—"}</TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
