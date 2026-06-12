import type { Game } from "@/types/cluster"
import { Badge } from "@/components/ui/badge"
import { MarketRow } from "./MarketRow"

interface GameCardProps {
  game: Game
}

export function GameCard({ game }: GameCardProps) {
  const platformColors: Record<string, string> = {
    betano: "bg-orange-100 text-orange-800 dark:bg-orange-900/40 dark:text-orange-300",
    lebull: "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300",
    bwin: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-300",
  }

  return (
    <div className="rounded-lg border p-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs text-muted-foreground">{game.competition}</span>
        <Badge
          variant="outline"
          className={platformColors[game.platform] ?? ""}
        >
          {game.platform}
        </Badge>
      </div>

      <div className="text-sm font-medium text-center">
        {game.home_team} vs {game.away_team}
      </div>

      <div className="space-y-2">
        {game.markets.slice(0, 3).map((m, i) => (
          <MarketRow key={i} market={m} />
        ))}
        {game.markets.length > 3 && (
          <p className="text-xs text-muted-foreground text-center">
            +{game.markets.length - 3} more markets
          </p>
        )}
      </div>
    </div>
  )
}
