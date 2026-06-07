const EPSILON = 0.001

export function approxEq(a: number, b: number): boolean {
  return Math.abs(a - b) < EPSILON
}

function approxGt(a: number, b: number): boolean {
  return a - b > EPSILON
}

import type { Cluster, Game, Market } from "@/types/cluster"
import type { ChangeDirection } from "@/stores/clusters-ui"

export function detectClusterChange(
  prev: Cluster,
  next: Cluster,
): ChangeDirection {
  const prevOdds = extractAllOdds(prev)
  const nextOdds = extractAllOdds(next)
  let hasUp = false
  let hasDown = false

  for (const [key, cur] of nextOdds) {
    const old = prevOdds.get(key)
    if (old !== undefined && !approxEq(old, cur)) {
      if (approxGt(cur, old)) hasUp = true
      else hasDown = true
    }
  }

  if (hasUp && hasDown) return "mixed"
  if (hasUp) return "up"
  if (hasDown) return "down"
  return null
}

function gameKey(game: Game): string {
  return `${game.platform}::${game.id}`
}

function extractAllOdds(cluster: Cluster): Map<string, number> {
  const odds = new Map<string, number>()
  for (const game of cluster.games) {
    const gk = gameKey(game)
    for (const market of game.markets) {
      extractMarketOdds(gk, market, odds)
    }
  }
  return odds
}

function extractMarketOdds(
  prefix: string,
  market: Market,
  acc: Map<string, number>,
): void {
  switch (market.type) {
    case "MatchResult":
      acc.set(`${prefix}/1`, market.home.value)
      acc.set(`${prefix}/X`, market.draw.value)
      acc.set(`${prefix}/2`, market.away.value)
      break
    case "Moneyline":
      acc.set(`${prefix}/1`, market.home.value)
      acc.set(`${prefix}/2`, market.away.value)
      break
    case "Total":
      acc.set(`${prefix}/over@${market.line}`, market.over.value)
      acc.set(`${prefix}/under@${market.line}`, market.under.value)
      break
    case "Handicap":
      acc.set(`${prefix}/1@${market.line}`, market.home.value)
      acc.set(`${prefix}/X@${market.line}`, market.draw.value)
      acc.set(`${prefix}/2@${market.line}`, market.away.value)
      break
    case "AsianHandicap":
      acc.set(`${prefix}/home@${market.line}`, market.home.value)
      acc.set(`${prefix}/away@${market.line}`, market.away.value)
      break
  }
}
