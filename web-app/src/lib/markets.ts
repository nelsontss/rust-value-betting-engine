import type { Game, Market } from "@/types/cluster"

export interface MarketGroupItem {
  platform: string
  gameId: string
  link: string | null
  market: Market
}

export interface MarketGroup {
  key: string
  label: string
  items: MarketGroupItem[]
}

const MARKET_ORDER: Record<string, number> = {
  MatchResult: 0,
  Moneyline: 1,
  DoubleChance: 2,
  Total: 3,
  Handicap: 4,
  AsianHandicap: 5,
}

function marketTypeOrder(key: string): number {
  const type = key.split("@")[0]
  return MARKET_ORDER[type] ?? 99
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
      group.items.push({ platform: game.platform, gameId: game.id, link: game.link, market })
    }
  }
  return [...map.values()].sort((a, b) => {
    const o = marketTypeOrder(a.key) - marketTypeOrder(b.key)
    return o !== 0 ? o : a.key.localeCompare(b.key)
  })
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
