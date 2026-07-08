import { useState } from "react"
import type { Cluster } from "@/types/cluster"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { MarketGroupTable } from "./MarketGroupTable"
import { MarketChart } from "./MarketChart"
import { groupMarkets } from "@/lib/markets"
import { useClusterMarketHistory } from "@/hooks/useClusterMarketHistory"

interface ClusterDetailProps {
  cluster: Cluster
}

export function ClusterDetail({ cluster }: ClusterDetailProps) {
  const allGroups = groupMarkets(cluster.games)
  const groups = allGroups.filter(
    (g) => new Set(g.items.map((i) => i.platform)).size >= 2,
  )

  const [selectedGroupKey, setSelectedGroupKey] = useState<string | null>(null)

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

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 p-4">
      <div className="space-y-0.5 overflow-y-auto max-h-[calc(100vh-8rem)] px-1">
        {groups.map((group) => (
          <div
            key={group.key}
            className={`rounded-lg border cursor-pointer transition-colors ${
              selectedGroupKey === group.key
                ? "ring-2 ring-primary"
                : "hover:bg-muted/50"
            }`}
            onClick={() => setSelectedGroupKey(group.key)}
          >
            <div className="px-2.5 py-2">
              <MarketGroupTable group={group} compact />
            </div>
          </div>
        ))}
      </div>

      <div className="space-y-4">
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
      </div>
    </div>
  )
}
