import type { Cluster } from "@/types/cluster"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { MarketGroupTable } from "./MarketGroupTable"
import { ExternalLink } from "lucide-react"
import { MarketChart } from "./MarketChart"
import { LiveDiffComparisonTable } from "./LiveDiffComparisonTable"
import { groupMarkets } from "@/lib/markets"
import { useClusterMarketHistory } from "@/hooks/useClusterMarketHistory"

interface ClusterInspectorProps {
  cluster: Cluster
  selectedGroupKey: string | null
  onSelectGroupKey: (key: string) => void
}

export function ClusterInspector({
  cluster,
  selectedGroupKey,
  onSelectGroupKey,
}: ClusterInspectorProps) {
  const allGroups = groupMarkets(cluster.games)
  const selectedGroup = selectedGroupKey
    ? allGroups.find((g) => g.key === selectedGroupKey) ?? null
    : null

  const selectedGames = selectedGroup
    ? cluster.games.filter((g) =>
        selectedGroup.items.some((i) => i.gameId === g.id),
      )
    : []

  const selectedMarketItem = selectedGroup?.items[0]
  const targetMarket =
    selectedMarketItem && "line" in selectedMarketItem.market
      ? { type: selectedMarketItem.market.type, line: selectedMarketItem.market.line }
      : selectedMarketItem
        ? { type: selectedMarketItem.market.type }
        : null

  const gamePlatforms = selectedGames.map((g) => ({
    id: g.id,
    platform: g.platform,
  }))

  const { data: marketHistory, isLoading: historyLoading } =
    useClusterMarketHistory(selectedGames, targetMarket?.type ?? "")

  const rep = cluster.representative_game
  const platformLinks = new Map<string, string | null>()
  cluster.games.forEach((g) => {
    if (!platformLinks.has(g.platform) && g.link) platformLinks.set(g.platform, g.link)
    else if (!platformLinks.has(g.platform)) platformLinks.set(g.platform, null)
  })
  const platforms = [...platformLinks.keys()]
  const multiPlatform = allGroups.filter(
    (g) => new Set(g.items.map((i) => i.platform)).size >= 2,
  )

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          {rep ? (
            <>
              <h2 className="text-lg font-semibold truncate">
                {rep.home_team} vs {rep.away_team}
              </h2>
              <p className="text-sm text-muted-foreground">
                {rep.competition} &middot; {rep.country} &middot;{" "}
                {new Date(rep.date).toLocaleString(undefined, {
                  weekday: "short",
                  month: "short",
                  day: "numeric",
                  hour: "2-digit",
                  minute: "2-digit",
                })}
              </p>
            </>
          ) : (
            <h2 className="text-lg font-semibold">{cluster.id}</h2>
          )}
        </div>
        <div className="flex items-center gap-1 shrink-0">
          {platforms.map((p) => {
            const link = platformLinks.get(p)
            return (
              <span key={p} className="inline-flex items-center gap-0.5">
                <Badge variant="secondary" className="text-xs">
                  {p}
                </Badge>
                {link && (
                  <a
                    href={link}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground"
                  >
                    <ExternalLink className="size-3.5" />
                  </a>
                )}
              </span>
            )
          })}
        </div>
      </div>

      {multiPlatform.length > 0 && (
        <p className="text-xs text-emerald-600 font-medium">
          {multiPlatform.length} market type
          {multiPlatform.length !== 1 ? "s" : ""} across {platforms.length}{" "}
          platforms
        </p>
      )}

      <div className="flex flex-col lg:flex-row gap-4 items-start">
        <div className="shrink-0 space-y-0.5">
          {allGroups.map((group) => {
            const groupPlatforms = new Set(
              group.items.map((i) => i.platform),
            ).size
            return (
              <div
                key={group.key}
                className={`rounded-lg border cursor-pointer transition-colors ${
                  selectedGroupKey === group.key
                    ? "ring-2 ring-primary"
                    : "hover:bg-muted/50"
                }`}
                onClick={() => onSelectGroupKey(group.key)}
              >
                <div className="px-2.5 py-2">
                  <MarketGroupTable group={group} compact />
                  {groupPlatforms >= 2 && (
                    <p className="text-[10px] text-emerald-600 mt-1">
                      {groupPlatforms} platforms
                    </p>
                  )}
                </div>
              </div>
            )
          })}
        </div>

        <div className="flex-1 min-w-0 space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Market History</CardTitle>
            </CardHeader>
            <CardContent>
              {selectedGroupKey && targetMarket ? (
                historyLoading ? (
                  <div className="text-center text-muted-foreground py-8">
                    Loading market history...
                  </div>
                ) : marketHistory ? (
                  <MarketChart
                    data={marketHistory}
                    targetMarket={targetMarket}
                    games={gamePlatforms}
                  />
                ) : null
              ) : (
                <div className="text-center text-muted-foreground py-8">
                  Select a market to view historical odds across platforms
                </div>
              )}
            </CardContent>
          </Card>
          <LiveDiffComparisonTable cluster={cluster} filterMarket={targetMarket} />
        </div>
      </div>
    </div>
  )
}
