import { useClusterStatistics } from "@/hooks/useClusterStatistics"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { OutcomeStatistics, StatisticsValues } from "@/types/statistics"
import { cn } from "@/lib/utils"

function fmt(value: number | null, digits = 3): string {
  if (value === null) return "–"
  return value.toFixed(digits)
}

function DiffCell({ value }: { value: number | null }) {
  if (value === null) {
    return <span className="text-muted-foreground">–</span>
  }
  const sign = value > 0 ? "+" : value < 0 ? "−" : ""
  return (
    <span
      className={cn(
        "font-mono tabular-nums font-medium",
        value > 0 && "text-emerald-600",
        value < 0 && "text-red-600",
      )}
    >
      {sign}
      {fmt(value)}
    </span>
  )
}

function OutcomeTable({
  outcomes,
}: {
  outcomes: OutcomeStatistics
}) {
  const entries = Object.entries(outcomes).sort(([a], [b]) => a.localeCompare(b))

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Outcome</TableHead>
          <TableHead className="text-right">Mean diff</TableHead>
          <TableHead className="text-right">Median diff</TableHead>
          <TableHead className="text-right">p25</TableHead>
          <TableHead className="text-right">p75</TableHead>
          <TableHead className="text-right">Samples</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {entries.map(([outcome, values]) => (
          <OutcomeRow key={outcome} outcome={outcome} values={values} />
        ))}
      </TableBody>
    </Table>
  )
}

function OutcomeRow({
  outcome,
  values,
}: {
  outcome: string
  values: StatisticsValues
}) {
  return (
    <TableRow>
      <TableCell className="font-medium">{outcome}</TableCell>
      <TableCell className="text-right">
        <DiffCell value={values.mean_diff} />
      </TableCell>
      <TableCell className="text-right">
        <DiffCell value={values.median_diff} />
      </TableCell>
      <TableCell className="text-right">
        <DiffCell value={values.p25_diff} />
      </TableCell>
      <TableCell className="text-right">
        <DiffCell value={values.p75_diff} />
      </TableCell>
      <TableCell className="text-right font-mono tabular-nums text-muted-foreground">
        {values.samples}
      </TableCell>
    </TableRow>
  )
}

export function StatisticsPage() {
  const { statistics, isConnected } = useClusterStatistics()

  if (!statistics) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-muted-foreground">
        Waiting for statistics...
      </div>
    )
  }

  const marketTypes = Object.entries(statistics).sort(([a], [b]) =>
    a.localeCompare(b),
  )

  return (
    <div>
      <header className="sticky top-14 z-10 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center justify-between px-4 h-14">
          <h1 className="text-lg font-semibold">Statistics</h1>
          <div className="flex items-center gap-2">
            <Badge
              variant={isConnected ? "default" : "destructive"}
              className="text-xs"
            >
              {isConnected ? "Live" : "Disconnected"}
            </Badge>
          </div>
        </div>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 p-4">
        {marketTypes.map(([marketType, outcomes]) => (
          <Card key={marketType}>
            <CardHeader>
              <CardTitle className="font-mono text-sm">{marketType}</CardTitle>
            </CardHeader>
            <CardContent>
              <OutcomeTable outcomes={outcomes} />
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}