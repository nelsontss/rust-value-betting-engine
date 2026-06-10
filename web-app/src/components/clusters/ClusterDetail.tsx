import type { Cluster } from "@/types/cluster"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { MarketGroupTable } from "./MarketGroupTable"
import { groupMarkets } from "@/lib/markets"

interface ClusterDetailProps {
  cluster: Cluster
}

export function ClusterDetail({ cluster }: ClusterDetailProps) {
  const rep = cluster.representative_game
  const platforms = [...new Set(cluster.games.map((g) => g.platform))]
  const groups = groupMarkets(cluster.games).filter(
    (g) => new Set(g.items.map((i) => i.platform)).size >= 2,
  )

  return (
    <div className="space-y-6 p-4">
      <Card>
        <CardHeader>
          {rep && (
            <div className="flex items-start justify-between">
              <div>
                <CardTitle className="text-xl">
                  {rep.home_team} vs {rep.away_team}
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  {rep.competition} &middot; {rep.country} &middot;{" "}
                  {new Date(rep.date).toLocaleDateString()}
                </p>
              </div>
              <div className="flex gap-1">
                {platforms.map((p) => (
                  <Badge key={p} variant="secondary">
                    {p}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </CardHeader>
        <CardContent className="space-y-1 text-sm text-muted-foreground">
          <p>Cluster ID: {cluster.id}</p>
          <p>Platforms: {cluster.games.length}</p>
          <p>Last updated: {new Date(cluster.updated_at).toLocaleString()}</p>
        </CardContent>
      </Card>

      <div className="grid gap-4">
        {groups.map((group) => (
          <Card key={group.key}>
            <CardContent className="pt-4">
              <MarketGroupTable group={group} />
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
