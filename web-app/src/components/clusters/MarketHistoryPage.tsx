import { useState } from "react"
import { useParams, Link } from "@tanstack/react-router"
import { useMarketHistory } from "@/hooks/useMarketHistory"
import { MarketChart } from "./MarketChart"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { ArrowLeft, ChevronLeft, ChevronRight } from "lucide-react"
import type { MarketData } from "@/types/market-history"

export function MarketHistoryPage() {
  const { gameId } = useParams({ from: "/market-history/$gameId" })
  const [selectedMarketIndex, setSelectedMarketIndex] = useState(0)
  const { data, isLoading, error } = useMarketHistory({ gameId })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-muted-foreground">
        Loading market history...
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-destructive">
        <div className="text-center">
          <p>Failed to load market history</p>
          <p className="text-sm text-muted-foreground mt-2">{error.message}</p>
        </div>
      </div>
    )
  }

  if (!data || data.markets.length === 0) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-muted-foreground">
        No market data available
      </div>
    )
  }

  const currentMarket = data.markets[selectedMarketIndex]?.market as MarketData | undefined

  return (
    <div>
      <header className="sticky top-14 z-10 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center justify-between px-4 h-14">
          <div className="flex items-center gap-2">
            <Link to="/games">
              <Button variant="ghost" size="sm">
                <ArrowLeft className="size-4 mr-1" />
                Back
              </Button>
            </Link>
            <h1 className="text-lg font-semibold">Market History</h1>
          </div>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <span>{data.markets.length} data points</span>
            {data.markets.length > 0 && (
              <span className="text-xs">
                (Live updates enabled)
              </span>
            )}
          </div>
        </div>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4 p-4 h-[calc(100vh-7rem)]">
        {/* Markets Sidebar */}
        <div className="lg:col-span-1 border rounded-lg p-4 overflow-y-auto">
          <div className="flex items-center justify-between mb-4">
            <h2 className="font-semibold text-sm">Markets</h2>
            <Badge variant="outline" className="text-xs">
              {selectedMarketIndex + 1} / {data.markets.length}
            </Badge>
          </div>

          <div className="space-y-2 mb-4">
            {data.markets.map((point, index) => {
              const market = point.market as MarketData
              const isSelected = index === selectedMarketIndex
              
              return (
                <button
                  key={index}
                  onClick={() => setSelectedMarketIndex(index)}
                  className={`w-full text-left p-2 rounded-md text-sm transition-colors ${
                    isSelected
                      ? "bg-primary text-primary-foreground"
                      : "bg-muted hover:bg-muted/80"
                  }`}
                >
                  <div className="font-medium text-xs">
                    {market.type}
                    {("line" in market) && ` (${market.line})`}
                  </div>
                  <div className="text-xs opacity-75 mt-1">
                    {new Date(point.timestamp).toLocaleTimeString()}
                  </div>
                </button>
              )
            })}
          </div>

          {/* Navigation Buttons */}
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() =>
                setSelectedMarketIndex((prev) =>
                  prev > 0 ? prev - 1 : data.markets.length - 1
                )
              }
              disabled={data.markets.length <= 1}
            >
              <ChevronLeft className="size-4" />
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() =>
                setSelectedMarketIndex((prev) =>
                  prev < data.markets.length - 1 ? prev + 1 : 0
                )
              }
              disabled={data.markets.length <= 1}
            >
              <ChevronRight className="size-4" />
            </Button>
          </div>
        </div>

        {/* Chart */}
        <div className="lg:col-span-3">
          {currentMarket ? (
            <MarketChart data={data} marketIndex={selectedMarketIndex} />
          ) : (
            <Card className="p-4 text-center text-muted-foreground h-full flex items-center justify-center">
              Loading chart...
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
