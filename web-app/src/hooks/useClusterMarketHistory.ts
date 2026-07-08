import { useEffect, useMemo, useState } from "react"
import type { MarketData, MarketDataPointResponse, MarketHistoryResponse } from "@/types/market-history"
import { apiUrl, sseUrl } from "@/lib/api"

export interface MarketHistoryPoint {
  timestamp: string
  gameId: string
  platform: string
  market: MarketData
}

export function useClusterMarketHistory(
  games: Array<{ id: string; platform: string }>,
  targetMarketType: string,
) {
  const [data, setData] = useState<MarketHistoryPoint[] | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  const gameKey = useMemo(
    () =>
      [...games]
        .sort((a, b) => a.id.localeCompare(b.id))
        .map((g) => `${g.id}:${g.platform}`)
        .join(","),
    [games],
  )

  useEffect(() => {
    if (!games.length) {
      setData(null)
      setIsLoading(false)
      setError(null)
      return
    }

    const platformMap = Object.fromEntries(games.map((g) => [g.id, g.platform]))
    const eventSources: EventSource[] = []
    let generation = 0
    let isMounted = true

    const init = async () => {
      const currentGen = ++generation
      setIsLoading(true)
      setError(null)

      try {
        const results = await Promise.all(
          games.map(async ({ id }) => {
            try {
              const res = await fetch(apiUrl(`/games/${id}/markets/history`))
              if (!res.ok) return [] as MarketHistoryPoint[]
              const history: MarketHistoryResponse = await res.json()
              return (history.markets_by_type[targetMarketType] ?? []).map((point) => ({
                timestamp: point.timestamp,
                gameId: id,
                platform: platformMap[id] ?? "",
                market: point.market,
              }))
            } catch {
              return [] as MarketHistoryPoint[]
            }
          }),
        )

        if (!isMounted || currentGen !== generation) return

        setData(results.flat())
        setIsLoading(false)

        for (const { id } of games) {
          const es = new EventSource(sseUrl(`/see/games/${id}/markets/history`))
          eventSources.push(es)

          es.onmessage = (event) => {
            try {
              const update: MarketDataPointResponse = JSON.parse(event.data)
              if (update.game_id === id && update.market.type === targetMarketType && isMounted) {
                setData((prev) => {
                  if (!prev) return prev
                  return [
                    ...prev,
                    {
                      timestamp: update.timestamp,
                      gameId: id,
                      platform: platformMap[id] ?? "",
                      market: update.market,
                    },
                  ]
                })
              }
            } catch (parseError) {
              console.error("Failed to parse SSE data:", parseError)
            }
          }

          es.onerror = () => {
            if (isMounted) {
              es.close()
            }
          }
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err : new Error(String(err)))
          setIsLoading(false)
        }
      }
    }

    init()

    return () => {
      isMounted = false
      generation++
      for (const es of eventSources) {
        es.close()
      }
    }
  }, [gameKey, targetMarketType])

  return { data, isLoading, error }
}
