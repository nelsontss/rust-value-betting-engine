import { useEffect, useRef } from "react"
import type { Cluster, Game } from "@/types/cluster"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Badge } from "@/components/ui/badge"
import { ExternalLink } from "lucide-react"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

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
  return [...clusters].sort(
    (a, b) => parseDate(repGame(a)?.date ?? "") - parseDate(repGame(b)?.date ?? ""),
  )
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
  const selectedRef = useRef<HTMLTableRowElement | null>(null)

  useEffect(() => {
    selectedRef.current?.scrollIntoView({ behavior: "smooth", block: "start" })
  }, [selectedId])

  return (
    <ScrollArea className="h-full">
      <Table className="table-fixed">
        <TableHeader className="sticky top-0 bg-background/95 backdrop-blur z-10">
          <TableRow className="hover:bg-transparent">
            <TableHead className="h-8 text-[11px] uppercase tracking-wide">
              Game
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((cluster) => {
            const rep = repGame(cluster)
            const name = rep ? `${rep.home_team} vs ${rep.away_team}` : cluster.id
            const league = rep?.competition?.trim() || "—"
            const time = parseDate(rep?.date ?? "")
            const isLive = time !== MAX_SAFE_DATE && time < Date.now() && time > Date.now() - 3 * 60 * 60 * 1000
            const selected = cluster.id === selectedId
            const platformLinks = new Map<string, string | null>()
            cluster.games.forEach((g) => {
              if (!platformLinks.has(g.platform) && g.link) platformLinks.set(g.platform, g.link)
              else if (!platformLinks.has(g.platform)) platformLinks.set(g.platform, null)
            })
            const platforms = [...platformLinks.keys()]
            return (
              <TableRow
                key={cluster.id}
                ref={selected ? selectedRef : undefined}
                data-state={selected ? "selected" : undefined}
                className={`cursor-pointer align-top ${selected ? "bg-accent border-l-2 border-l-primary" : ""}`}
                onClick={() => onSelect(cluster.id)}
              >
                <TableCell className="relative">
                  {isLive && (
                    <span className="absolute bottom-1.5 right-1.5 bg-red-600 text-white text-[9px] font-bold px-1.5 py-0.5 rounded leading-none">
                      LIVE
                    </span>
                  )}
                  <div className="font-medium line-clamp-2 leading-snug pr-8">
                    {name}
                  </div>
                  <div className="text-[11px] text-muted-foreground line-clamp-1 leading-snug">
                    {league} · {formatDate(time)}
                  </div>
                  {platforms.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-1">
                      {platforms.map((p) => {
                        const link = platformLinks.get(p)
                        return (
                          <span key={p} className="inline-flex items-center gap-0.5">
                            <Badge
                              variant="secondary"
                              className="text-[10px] h-4 px-1.5 font-mono"
                            >
                              {p}
                            </Badge>
                            {link && (
                              <a
                                href={link}
                                target="_blank"
                                rel="noopener noreferrer"
                                onClick={(e) => e.stopPropagation()}
                                className="p-0.5 rounded hover:bg-muted text-muted-foreground hover:text-foreground"
                              >
                                <ExternalLink className="size-3" />
                              </a>
                            )}
                          </span>
                        )
                      })}
                    </div>
                  )}
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </ScrollArea>
  )
}
