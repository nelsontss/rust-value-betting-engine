import type { Cluster, Game } from "@/types/cluster"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { cn } from "@/lib/utils"

interface ClusterTableProps {
  clusters: Cluster[]
  selectedId: string | null
  onSelect: (id: string) => void
}

const MAX_SAFE_DATE = Number.MAX_SAFE_INTEGER

function parseDate(dateStr: string): number {
  if (!dateStr) return MAX_SAFE_DATE
  const normalized = dateStr.includes("T")
    ? dateStr
    : dateStr.replace(" ", "T") + "Z"
  const time = new Date(normalized).getTime()
  return Number.isNaN(time) ? MAX_SAFE_DATE : time
}

function repGame(cluster: Cluster): Game | null {
  return cluster.representative_game
}

function sortNextFirst(clusters: Cluster[]): Cluster[] {
  const now = Date.now()
  const withTime = clusters.map((c) => ({
    cluster: c,
    time: parseDate(repGame(c)?.date ?? ""),
  }))
  const future = withTime
    .filter((x) => x.time !== MAX_SAFE_DATE && x.time >= now)
    .sort((a, b) => a.time - b.time)
  const past = withTime
    .filter((x) => x.time === MAX_SAFE_DATE || x.time < now)
    .sort((a, b) => b.time - a.time)
  return [...future, ...past].map((x) => x.cluster)
}

function formatDate(ts: number): string {
  if (ts === MAX_SAFE_DATE) return "—"
  return new Date(ts).toLocaleString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

export function ClusterTable({ clusters, selectedId, onSelect }: ClusterTableProps) {
  const rows = sortNextFirst(clusters)

  return (
    <ScrollArea className="h-full">
      <Table className="table-fixed">
        <TableHeader className="sticky top-0 bg-background/95 backdrop-blur z-10">
          <TableRow className="hover:bg-transparent">
            <TableHead className="h-8 text-[11px] uppercase tracking-wide">
              Game
            </TableHead>
            <TableHead className="h-8 text-[11px] uppercase tracking-wide w-[132px]">
              Date
            </TableHead>
            <TableHead className="h-8 text-[11px] uppercase tracking-wide w-[150px]">
              League
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((cluster) => {
            const rep = repGame(cluster)
            const name = rep ? `${rep.home_team} vs ${rep.away_team}` : cluster.id
            const league = rep?.competition?.trim() || "—"
            const time = parseDate(rep?.date ?? "")
            const isPast = time !== MAX_SAFE_DATE && time < Date.now()
            const selected = cluster.id === selectedId
            const platforms = [...new Set(cluster.games.map((g) => g.platform))]
            return (
              <TableRow
                key={cluster.id}
                data-state={selected ? "selected" : undefined}
                className="cursor-pointer align-top"
                onClick={() => onSelect(cluster.id)}
              >
                <TableCell className={cn(isPast && "opacity-60")}>
                  <div className="font-medium line-clamp-3 leading-snug">
                    {name}
                  </div>
                  {platforms.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-1">
                      {platforms.map((p) => (
                        <Badge
                          key={p}
                          variant="secondary"
                          className="text-[10px] h-4 px-1.5 font-mono"
                        >
                          {p}
                        </Badge>
                      ))}
                    </div>
                  )}
                </TableCell>
                <TableCell
                  className={cn(
                    "whitespace-nowrap text-xs",
                    isPast && "opacity-60",
                  )}
                >
                  {formatDate(time)}
                </TableCell>
                <TableCell
                  className={cn(
                    "text-muted-foreground text-xs line-clamp-2 leading-snug",
                    isPast && "opacity-60",
                  )}
                >
                  {league}
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </ScrollArea>
  )
}
