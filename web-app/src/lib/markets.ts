import type { Game, Market } from "@/types/cluster"

export interface MarketGroupItem {
  platform: string
  market: Market
}

export interface MarketGroup {
  key: string
  label: string
  items: MarketGroupItem[]
}

export function groupMarkets(games: Game[]): MarketGroup[] {
  const map = new Map<string, MarketGroup>()
  for (const game of games) {
    for (const market of game.markets) {
      const key = marketKey(market)
      let group = map.get(key)
      if (!group) {
        group = { key, label: marketLabel(market), items: [] }
        map.set(key, group)
      }
      group.items.push({ platform: game.platform, market })
    }
  }
  return [...map.values()]
}

function marketKey(market: Market): string {
  switch (market.type) {
    case "MatchResult":
      return "MatchResult"
    case "Moneyline":
      return "Moneyline"
    case "DoubleChance":
      return "DoubleChance"
    case "Total":
      return `Total@${market.line}`
    case "Handicap":
      return `Handicap@${market.line}`
    case "AsianHandicap":
      return `AsianHandicap@${market.line}`
  }
}

function marketLabel(market: Market): string {
  switch (market.type) {
    case "MatchResult":
      return "Match Result"
    case "Moneyline":
      return "Moneyline"
    case "DoubleChance":
      return "Double Chance"
    case "Total":
      return `Total O/U ${market.line}`
    case "Handicap":
      return `Handicap ${market.line >= 0 ? "+" : ""}${market.line}`
    case "AsianHandicap":
      return `Asian HCP ${market.line >= 0 ? "+" : ""}${market.line}`
  }
}
