import { useEffect, useState } from "react"
import type { MarketHistoryResponse, MarketDataPointResponse, MarketDataPoint } from "@/types/market-history"
import { apiUrl, sseUrl } from "@/lib/api"

interface UseMarketHistoryOptions {
  gameId: string
  enabled?: boolean
}

interface UseMarketHistoryResult {
  data: MarketHistoryResponse | null
  isLoading: boolean
  error: Error | null
}

export function useMarketHistory({
  gameId,
  enabled = true,
}: UseMarketHistoryOptions): UseMarketHistoryResult {
  const [data, setData] = useState<MarketHistoryResponse | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    if (!enabled || !gameId) {
      setIsLoading(false)
      return
    }

    let isMounted = true
    let eventSource: EventSource | null = null

    const fetchHistoricalData = async () => {
      try {
        setIsLoading(true)
        setError(null)
        
        const res = await fetch(apiUrl(`/games/${gameId}/markets/history`))
        if (!res.ok) {
          throw new Error(`Failed to fetch market history: ${res.status}`)
        }
        
        const historicalData: MarketHistoryResponse = await res.json()
        
        if (isMounted) {
          setData(historicalData)
          setIsLoading(false)
        }
        
        // After fetching historical data, set up SSE
        subscribeToUpdates(historicalData)
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err : new Error(String(err)))
          setIsLoading(false)
        }
      }
    }

    const subscribeToUpdates = (initialData: MarketHistoryResponse) => {
      try {
        eventSource = new EventSource(sseUrl(`/see/games/${gameId}/markets/history`))
        
        eventSource.onmessage = (event) => {
          try {
            const update: MarketDataPointResponse = JSON.parse(event.data)
            
            // Only process updates for the current game
            if (update.game_id === gameId && isMounted) {
              setData((prevData) => {
                if (!prevData) return initialData
                
                return {
                  ...prevData,
                  markets: [
                    ...prevData.markets,
                    {
                      timestamp: update.timestamp,
                      market: update.market,
                    } as MarketDataPoint,
                  ],
                }
              })
            }
          } catch (parseError) {
            console.error("Failed to parse SSE data:", parseError)
          }
        }
        
        eventSource.onerror = () => {
          if (isMounted && eventSource) {
            eventSource.close()
            setError(new Error("SSE connection lost"))
          }
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err : new Error(String(err)))
        }
      }
    }

    fetchHistoricalData()

    return () => {
      isMounted = false
      if (eventSource) {
        eventSource.close()
      }
    }
  }, [gameId, enabled])

  return { data, isLoading, error }
}
