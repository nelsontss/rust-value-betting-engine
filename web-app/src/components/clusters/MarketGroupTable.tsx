import type { Market } from "@/types/cluster"
import type { MarketGroup } from "@/lib/markets"
import { Badge } from "@/components/ui/badge"

interface MarketGroupTableProps {
  group: MarketGroup
  compact?: boolean
}

export function MarketGroupTable({ group, compact }: MarketGroupTableProps) {
  const first = group.items[0]?.market
  if (!first) return null

  const headers = getHeaders(first)
  const colClass = compact ? "w-10 text-center" : "w-14 text-center"

  return (
    <div className="space-y-1">
      <div className="flex items-center gap-2 text-xs">
        <span className="font-medium text-foreground">{group.label}</span>
        <div className="flex">
          {headers.map((h) => (
            <span key={h} className={`text-muted-foreground ${colClass}`}>
              {h}
            </span>
          ))}
        </div>
      </div>
      {group.items.map((item) => (
        <div key={item.platform} className="flex items-center gap-2">
          <Badge variant="outline" className="text-[10px] h-4 px-1 font-mono">
            {item.platform}
          </Badge>
          {getValues(item.market).map((v, i) => (
            <OddsValue key={i} value={v} className={colClass} />
          ))}
        </div>
      ))}
    </div>
  )
}

function OddsValue({ value, className }: { value: number; className?: string }) {
  return (
    <span className={`font-mono text-sm tabular-nums ${className ?? ""}`}>
      {value.toFixed(2)}
    </span>
  )
}

function getHeaders(market: Market): string[] {
  switch (market.type) {
    case "MatchResult":
      return ["1", "X", "2"]
    case "Moneyline":
      return ["1", "2"]
    case "Total":
      return ["Over", "Under"]
    case "Handicap":
      return ["1", "X", "2"]
    case "AsianHandicap":
      return ["Home", "Away"]
  }
}

function getValues(market: Market): number[] {
  switch (market.type) {
    case "MatchResult":
      return [market.home.value, market.draw.value, market.away.value]
    case "Moneyline":
      return [market.home.value, market.away.value]
    case "Total":
      return [market.over.value, market.under.value]
    case "Handicap":
      return [market.home.value, market.draw.value, market.away.value]
    case "AsianHandicap":
      return [market.home.value, market.away.value]
  }
}
