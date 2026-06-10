import type { Cluster } from "@/types/cluster"
import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { MarketGroupTable } from "./MarketGroupTable"
import { useClustersUIStore } from "@/stores/clusters-ui"
import { groupMarkets } from "@/lib/markets"

interface ClusterCardProps {
  cluster: Cluster
}

const borderStyles: Record<string, string> = {
  up: "border-emerald-500/60 shadow-[0_0_0_1px] shadow-emerald-500/30",
  down: "border-red-500/60 shadow-[0_0_0_1px] shadow-red-500/30",
  mixed: "border-amber-500/60 shadow-[0_0_0_1px] shadow-amber-500/30",
}

const indicatorIcons: Record<string, string> = {
  up: "\u25B2",
  down: "\u25BC",
  mixed: "\u25C6",
}

const indicatorColors: Record<string, string> = {
  up: "text-emerald-500",
  down: "text-red-500",
  mixed: "text-amber-500",
}

const MAX_GROUPS = 3

export function ClusterCard({ cluster }: ClusterCardProps) {
  const setSelectedClusterId = useClustersUIStore((s) => s.setSelectedClusterId)
  const change = useClustersUIStore((s) => s.clusterChanges[cluster.id])
  const rep = cluster.representative_game
  const platforms = [...new Set(cluster.games.map((g) => g.platform))]
  const timeAgo = getTimeAgo(cluster.updated_at)
  const groups = groupMarkets(cluster.games).filter(
    (g) => new Set(g.items.map((i) => i.platform)).size >= 2,
  )

  return (
    <Card
      className={`cursor-pointer transition-all hover:shadow-md ${
        change ? borderStyles[change] : ""
      }`}
      onClick={() => setSelectedClusterId(cluster.id)}
    >
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            {rep && (
              <>
                <h3 className="text-base font-semibold truncate">
                  {rep.home_team} vs {rep.away_team}
                </h3>
                <p className="text-xs text-muted-foreground">
                  {rep.competition} &middot; {rep.country}
                </p>
              </>
            )}
            {!rep && <h3 className="text-base font-semibold">{cluster.id}</h3>}
          </div>
          <div className="flex items-center gap-1 shrink-0">
            {change && (
              <span
                className={`text-xs font-bold ${indicatorColors[change]}`}
                title={`Odds moved ${change}`}
              >
                {indicatorIcons[change]}
              </span>
            )}
            {platforms.map((p) => (
              <Badge key={p} variant="secondary" className="text-xs">
                {p}
              </Badge>
            ))}
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-2">
        <div className="flex gap-2 text-xs text-muted-foreground">
          <span>
            {groups.length} market type{groups.length !== 1 ? "s" : ""}
          </span>
          <span>&middot;</span>
          <span>{timeAgo}</span>
        </div>

        <div className="space-y-2">
          {groups.slice(0, MAX_GROUPS).map((group) => (
            <div key={group.key} className="rounded-lg border p-2 space-y-1">
              <MarketGroupTable group={group} compact />
            </div>
          ))}
          {groups.length > MAX_GROUPS && (
            <p className="text-xs text-muted-foreground text-center pt-1">
              +{groups.length - MAX_GROUPS} more market types
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function getTimeAgo(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime()
  const secs = Math.floor(diff / 1000)
  if (secs < 60) return `${secs}s ago`
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  return `${hrs}h ago`
}
