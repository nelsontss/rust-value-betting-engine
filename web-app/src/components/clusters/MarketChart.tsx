import { useMemo } from "react"
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts"
import type { MarketHistoryResponse, MarketData } from "@/types/market-history"
import { Card } from "@/components/ui/card"

interface MarketChartProps {
  data: MarketHistoryResponse
  marketIndex: number
}

interface ChartDataPoint {
  timestamp: string
  [key: string]: string | number
}

export function MarketChart({ data, marketIndex }: MarketChartProps) {
  const chartData = useMemo(() => {
    if (!data.markets[marketIndex]) return []

    const targetMarket = data.markets[marketIndex]
    const relevantMarkets = data.markets.filter((point) => {
      const market = point.market as MarketData
      return market.type === targetMarket.market.type
    })

    return relevantMarkets.map((point) => {
      const market = point.market as MarketData
      const base: ChartDataPoint = {
        timestamp: new Date(point.timestamp).toLocaleTimeString(),
      }

      switch (market.type) {
        case "MatchResult":
          return {
            ...base,
            home: market.home.value,
            draw: market.draw.value,
            away: market.away.value,
          }
        case "Moneyline":
          return {
            ...base,
            home: market.home.value,
            away: market.away.value,
          }
        case "DoubleChance":
          return {
            ...base,
            homeOrDraw: market.home_or_draw.value,
            homeOrAway: market.home_or_away.value,
            drawOrAway: market.draw_or_away.value,
          }
        case "Total":
          return {
            ...base,
            over: market.over.value,
            under: market.under.value,
          }
        case "Handicap":
          return {
            ...base,
            home: market.home.value,
            draw: market.draw.value,
            away: market.away.value,
          }
        case "AsianHandicap":
          return {
            ...base,
            home: market.home.value,
            away: market.away.value,
          }
        default:
          return base
      }
    })
  }, [data, marketIndex])

  if (chartData.length === 0) {
    return (
      <Card className="p-4 text-center text-muted-foreground">
        No data available for this market
      </Card>
    )
  }

  const market = data.markets[marketIndex]?.market as MarketData | undefined
  if (!market) return null

  return (
    <Card className="p-4">
      <div className="mb-4">
        <h3 className="text-lg font-semibold">
          {market.type}
          {("line" in market) && ` (${market.line})`}
        </h3>
      </div>
      <ResponsiveContainer width="100%" height={400}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="timestamp" angle={-45} textAnchor="end" height={80} />
          <YAxis />
          <Tooltip />
          <Legend />
          {getLineComponents(market.type)}
        </LineChart>
      </ResponsiveContainer>
    </Card>
  )
}

function getLineComponents(marketType: string) {
  const colors = {
    home: "#ef4444",
    draw: "#f59e0b",
    away: "#22c55e",
    homeOrDraw: "#3b82f6",
    homeOrAway: "#8b5cf6",
    drawOrAway: "#ec4899",
    over: "#06b6d4",
    under: "#6366f1",
  }

  const lines: Record<string, string[]> = {
    MatchResult: ["home", "draw", "away"],
    Moneyline: ["home", "away"],
    DoubleChance: ["homeOrDraw", "homeOrAway", "drawOrAway"],
    Total: ["over", "under"],
    Handicap: ["home", "draw", "away"],
    AsianHandicap: ["home", "away"],
  }

  const marketLines = lines[marketType] || []

  return marketLines.map((line) => (
    <Line
      key={line}
      type="monotone"
      dataKey={line}
      stroke={colors[line as keyof typeof colors]}
      dot={false}
    />
  ))
}
