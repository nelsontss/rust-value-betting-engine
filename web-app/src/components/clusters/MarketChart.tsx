import { useCallback, useMemo, useState, type ReactNode } from "react"
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
import { Button } from "@/components/ui/button"
import type { MarketHistoryPoint } from "@/hooks/useClusterMarketHistory"

interface MarketChartProps {
  data: MarketHistoryPoint[]
  targetMarket: { type: string; line?: number }
  games: Array<{ id: string; platform: string }>
}

interface ChartDataPoint {
  timestamp: string
  [key: string]: string | number
}

const INTERVALS = [
  { label: "1H", duration: 60 * 60 * 1000 },
  { label: "3H", duration: 3 * 60 * 60 * 1000 },
  { label: "6H", duration: 6 * 60 * 60 * 1000 },
  { label: "1D", duration: 24 * 60 * 60 * 1000 },
  { label: "3D", duration: 3 * 24 * 60 * 60 * 1000 },
] as const

type IntervalLabel = (typeof INTERVALS)[number]["label"]

function slugify(s: string): string {
  return s.toLowerCase().replace(/[^a-z0-9]/g, "_")
}

export function MarketChart({ data, targetMarket, games }: MarketChartProps) {
  const [hiddenLines, setHiddenLines] = useState<Set<string>>(new Set())
  const [selectedInterval, setSelectedInterval] = useState<IntervalLabel>("1D")

  const allLineKeys = useMemo(() => {
    const prefixList = [...new Set(games.map((g) => slugify(g.platform)))]
    const oddKeys: Record<string, string[]> = {
      MatchResult: ["home", "draw", "away"],
      Moneyline: ["home", "away"],
      DoubleChance: ["homeOrDraw", "homeOrAway", "drawOrAway"],
      Total: ["over", "under"],
      Handicap: ["home", "draw", "away"],
      AsianHandicap: ["home", "away"],
    }
    const keysForType = oddKeys[targetMarket.type] || []
    return prefixList.flatMap((p) => keysForType.map((k) => `${p}_${k}`))
  }, [games, targetMarket.type])

  const toggleLines = useCallback((keys: string[], isolate: boolean) => {
    setHiddenLines((prev) => {
      if (isolate) {
        const allHidden = new Set(allLineKeys)
        for (const k of keys) allHidden.delete(k)
        const prevArr = [...prev]
        const match = prevArr.length === allHidden.size && prevArr.every((k) => allHidden.has(k))
        if (match) return new Set()
        const result = new Set(allLineKeys)
        for (const k of keys) result.delete(k)
        return result
      }
      const next = new Set(prev)
      for (const k of keys) {
        if (next.has(k)) next.delete(k)
        else next.add(k)
      }
      if (next.size === 0 || next.size === allLineKeys.length) return new Set()
      return next
    })
  }, [allLineKeys])

  const handleLegendPlatform = useCallback((platform: string, meta: boolean) => {
    const prefix = slugify(platform)
    const keys = allLineKeys.filter((k) => k.startsWith(`${prefix}_`))
    toggleLines(keys, !meta)
  }, [allLineKeys, toggleLines])

  const handleLegendItem = useCallback((dataKey: string, meta: boolean) => {
    toggleLines([dataKey], !meta)
  }, [toggleLines])
  const intervalCutoff = useMemo(() => {
    const interval = INTERVALS.find((i) => i.label === selectedInterval)!
    return Date.now() - interval.duration
  }, [selectedInterval])

  const chartData = useMemo(() => {
    const filtered = data.filter((point) => {
      const ts = new Date(point.timestamp).getTime()
      if (ts < intervalCutoff) return false
      if ("line" in targetMarket && "line" in point.market) {
        return point.market.line === targetMarket.line
      }
      return true
    })

    const timeFmt = new Intl.DateTimeFormat([], { hour: "2-digit", minute: "2-digit", hour12: false })
    const dayTimeFmt = new Intl.DateTimeFormat([], {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    })

    const groups = new Map<number, ChartDataPoint & { _ts: number }>()
    const order: number[] = []

    for (const point of filtered) {
      const ts = new Date(point.timestamp).getTime()
      const key = Math.floor(ts / 60_000) * 60_000
      const prefix = slugify(point.platform)

      if (!groups.has(key)) {
        groups.set(key, { timestamp: "", _ts: key })
        order.push(key)
      }
      const row = groups.get(key)!
      const m = point.market

      switch (m.type) {
        case "MatchResult":
          row[`${prefix}_home`] = m.home.value
          row[`${prefix}_draw`] = m.draw.value
          row[`${prefix}_away`] = m.away.value
          break
        case "Moneyline":
          row[`${prefix}_home`] = m.home.value
          row[`${prefix}_away`] = m.away.value
          break
        case "DoubleChance":
          row[`${prefix}_homeOrDraw`] = m.home_or_draw.value
          row[`${prefix}_homeOrAway`] = m.home_or_away.value
          row[`${prefix}_drawOrAway`] = m.draw_or_away.value
          break
        case "Total":
          row[`${prefix}_over`] = m.over.value
          row[`${prefix}_under`] = m.under.value
          break
        case "Handicap":
          row[`${prefix}_home`] = m.home.value
          row[`${prefix}_draw`] = m.draw.value
          row[`${prefix}_away`] = m.away.value
          break
        case "AsianHandicap":
          row[`${prefix}_home`] = m.home.value
          row[`${prefix}_away`] = m.away.value
          break
      }
    }

    order.sort((a, b) => a - b)

    const last = new Map<string, number>()
    for (const ts of order) {
      const row = groups.get(ts)!
      for (const k of allLineKeys) {
        const v = row[k]
        if (v !== undefined) last.set(k, v as number)
        else if (last.has(k)) row[k] = last.get(k)!
      }
    }

    const spansMultipleDays =
      new Set(
        order.map((ts) => new Intl.DateTimeFormat([], { dateStyle: "short" }).format(new Date(ts))),
      ).size > 1

    return order.map((k) => {
      const row = groups.get(k)!
      row.timestamp = (spansMultipleDays ? dayTimeFmt : timeFmt).format(new Date(k))
      return row
    })
  }, [data, targetMarket, allLineKeys, intervalCutoff])

  const yDomain = useMemo<[number, number]>(() => {
    const visibleKeys =
      hiddenLines.size > 0 ? allLineKeys.filter((k) => !hiddenLines.has(k)) : allLineKeys
    const findExtents = (keys: string[]) => {
      let lo = Infinity
      let hi = -Infinity
      for (const row of chartData) {
        for (const k of keys) {
          const v = row[k]
          if (typeof v === "number" && Number.isFinite(v)) {
            if (v < lo) lo = v
            if (v > hi) hi = v
          }
        }
      }
      return { lo, hi }
    }
    let { lo: min, hi: max } = findExtents(visibleKeys)
    if (!Number.isFinite(min) || !Number.isFinite(max)) {
      const fallback = findExtents(allLineKeys)
      min = fallback.lo
      max = fallback.hi
    }
    if (!Number.isFinite(min) || !Number.isFinite(max)) return [1.01, 5]
    if (min === max) {
      const pad = Math.max(Math.abs(min) * 0.06, 0.1)
      return [Math.max(1.01, min - pad), max + pad]
    }
    const pad = (max - min) * 0.14
    const lo = Math.max(1.01, min - pad)
    const hi = max + pad
    if (hi - lo < 0.05) return [Math.max(1.01, lo - 0.05), hi + 0.05]
    return [lo, hi]
  }, [chartData, hiddenLines, allLineKeys])

  if (chartData.length === 0) {
    const hasAnyData = data.some((point) => {
      if ("line" in targetMarket && "line" in point.market) {
        return point.market.line === targetMarket.line
      }
      return true
    })
    return (
      <div className="text-center text-muted-foreground py-8">
        {hasAnyData
          ? `No data in the last ${selectedInterval}`
          : "No historical data available for this market"}
      </div>
    )
  }

  const marketLabel =
    targetMarket.type +
    ("line" in targetMarket ? ` (${targetMarket.line})` : "")

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">{marketLabel}</h3>
        <div className="flex gap-1">
          {INTERVALS.map((interval) => (
            <Button
              key={interval.label}
              variant={selectedInterval === interval.label ? "default" : "ghost"}
              size="xs"
              onClick={() => setSelectedInterval(interval.label)}
            >
              {interval.label}
            </Button>
          ))}
        </div>
      </div>
      <ResponsiveContainer width="100%" height={400}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
          <XAxis dataKey="timestamp" tickCount={5} height={30} tick={{ fontSize: 11 }} />
          <YAxis
            type="number"
            domain={yDomain}
            allowDecimals
            allowDataOverflow={false}
            tickFormatter={(v: number) => (v >= 10 ? v.toFixed(1) : v.toFixed(2))}
            tick={{ fontSize: 11 }}
            width={52}
          />
          <Tooltip
            formatter={(value) => (typeof value === "number" ? value.toFixed(2) : String(value ?? ""))}
          />
          <Legend
            content={<LegendGrouped onPlatformClick={handleLegendPlatform} onItemClick={handleLegendItem} hiddenLines={hiddenLines} />}
          />
          {getLineComponents(games, targetMarket.type, hiddenLines)}
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}

const labelMap: Record<string, string> = {
  home: "Home",
  away: "Away",
  draw: "Draw",
  homeOrDraw: "1X",
  homeOrAway: "12",
  drawOrAway: "X2",
  over: "Over",
  under: "Under",
}

function LegendGrouped({
  payload,
  onPlatformClick,
  onItemClick,
  hiddenLines,
}: {
  payload?: Array<{ value: string; color: string; strokeDasharray?: string }>
  onPlatformClick: (platform: string, meta: boolean) => void
  onItemClick: (dataKey: string, meta: boolean) => void
  hiddenLines: Set<string>
}) {
  const groups = useMemo(() => {
    const map = new Map<string, Array<{ value: string; color: string; dataKey: string }>>()
    if (!payload) return map
    for (const entry of payload) {
      const spaceIdx = entry.value.lastIndexOf(" ")
      const platform = entry.value.slice(0, spaceIdx)
      const key = entry.value.slice(spaceIdx + 1)
      if (!map.has(platform)) map.set(platform, [])
      const dataKey = `${slugify(platform)}_${key}`
      map.get(platform)!.push({ color: entry.color, value: labelMap[key] ?? key, dataKey })
    }
    return map
  }, [payload])

  const allHidden = hiddenLines.size > 0

  const items: ReactNode[] = []
  for (const [platform, entries] of groups) {
    const platformHidden = allHidden && entries.every((e) => hiddenLines.has(e.dataKey))
    const partial = allHidden && entries.some((e) => hiddenLines.has(e.dataKey)) && !platformHidden
    items.push(
      <div key={platform} className="flex flex-col gap-1 min-w-[120px]">
        <span
          className={`text-xs font-semibold cursor-pointer select-none ${platformHidden ? "text-muted-foreground/40 line-through" : partial ? "text-muted-foreground" : "text-foreground"}`}
          onClick={(e) => onPlatformClick(platform, e.metaKey || e.ctrlKey)}
        >
          {platform}
        </span>
        {entries.map((e) => {
          const hidden = hiddenLines.has(e.dataKey)
          return (
            <div
              key={e.value}
              className={`flex items-center gap-1.5 text-xs cursor-pointer select-none ${hidden ? "text-muted-foreground/40 line-through" : "text-muted-foreground"}`}
              onClick={(ev) => onItemClick(e.dataKey, ev.metaKey || ev.ctrlKey)}
            >
              <span className="inline-block w-2.5 h-2.5 rounded-full" style={{ background: e.color }} />
              <span>{e.value}</span>
            </div>
          )
        })}
      </div>,
    )
  }

  return <div className="flex flex-wrap gap-4 pt-2 justify-center">{items}</div>
}

function getLineComponents(
  games: Array<{ platform: string }>,
  marketType: string,
  hiddenLines: Set<string>,
) {
  const platformColors = [
    "#3b82f6",
    "#ef4444",
    "#22c55e",
    "#f59e0b",
    "#a855f7",
    "#ec4899",
    "#06b6d4",
    "#f97316",
  ]

  const oddKeys: Record<string, string[]> = {
    MatchResult: ["home", "draw", "away"],
    Moneyline: ["home", "away"],
    DoubleChance: ["homeOrDraw", "homeOrAway", "drawOrAway"],
    Total: ["over", "under"],
    Handicap: ["home", "draw", "away"],
    AsianHandicap: ["home", "away"],
  }

  const keysForType = oddKeys[marketType] || []

  function slugify(s: string): string {
    return s.toLowerCase().replace(/[^a-z0-9]/g, "_")
  }

  const deduped = [...new Map(games.map((g) => [slugify(g.platform), g])).values()]
  return deduped.flatMap((game, gameIdx) => {
    const prefix = slugify(game.platform)
    const color = platformColors[gameIdx % platformColors.length]

    return keysForType.map((key) => {
      const dataKey = `${prefix}_${key}`
      return (
        <Line
          key={dataKey}
          type="monotone"
          dataKey={dataKey}
          stroke={color}
          connectNulls
          dot={false}
          isAnimationActive={false}
          hide={hiddenLines.has(dataKey)}
          name={`${game.platform} ${key}`}
        />
      )
    })
  })
}
