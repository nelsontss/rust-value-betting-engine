import type { Market } from "@/types/cluster"
import { OddsCell } from "./OddsCell"

interface MarketRowProps {
  market: Market
}

export function MarketRow({ market }: MarketRowProps) {
  switch (market.type) {
    case "MatchResult":
      return (
        <div className="flex gap-4 justify-center">
          <OddsCell label="1" odd={market.home} />
          <OddsCell label="X" odd={market.draw} />
          <OddsCell label="2" odd={market.away} />
        </div>
      )
    case "Moneyline":
      return (
        <div className="flex gap-4 justify-center">
          <OddsCell label="1" odd={market.home} />
          <OddsCell label="2" odd={market.away} />
        </div>
      )
    case "Total":
      return (
        <div className="flex gap-4 justify-center">
          <span className="flex items-center text-xs text-muted-foreground font-mono">
            O/U {market.line}
          </span>
          <OddsCell label="Over" odd={market.over} />
          <OddsCell label="Under" odd={market.under} />
        </div>
      )
    case "Handicap":
      return (
        <div className="flex gap-4 justify-center">
          <span className="flex items-center text-xs text-muted-foreground font-mono">
            H{market.line >= 0 ? "+" : ""}
            {market.line}
          </span>
          <OddsCell label="1" odd={market.home} />
          <OddsCell label="X" odd={market.draw} />
          <OddsCell label="2" odd={market.away} />
        </div>
      )
    case "AsianHandicap":
      return (
        <div className="flex gap-4 justify-center">
          <span className="flex items-center text-xs text-muted-foreground font-mono">
            AH {market.line >= 0 ? "+" : ""}
            {market.line}
          </span>
          <OddsCell label="Home" odd={market.home} />
          <OddsCell label="Away" odd={market.away} />
        </div>
      )
  }
}
