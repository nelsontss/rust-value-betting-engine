import { Link, useParams } from "@tanstack/react-router"
import { usePlatformGames } from "@/hooks/useGames"
import { GameCard } from "./GameCard"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ArrowLeft } from "lucide-react"

const ALL_PLATFORMS = ["betano", "lebull"]

export function PlatformGamesPage() {
  const { platform } = useParams({ from: "/games/$platform" })
  const { data: games, isLoading, error } = usePlatformGames(platform)

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-muted-foreground">
        Loading {platform} games...
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-destructive">
        Failed to load games: {(error as Error).message}
      </div>
    )
  }

  return (
    <div>
      <header className="sticky top-14 z-10 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center justify-between px-4 h-14">
          <div className="flex items-center gap-2">
            <Link to="/games">
              <Button variant="ghost" size="sm">
                <ArrowLeft className="size-4 mr-1" />
                All Games
              </Button>
            </Link>
            <h1 className="text-lg font-semibold capitalize">{platform}</h1>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">
              {games?.length ?? 0} games
            </span>
            <div className="flex gap-1 ml-2">
              {ALL_PLATFORMS.map((p) => (
                <Link key={p} to="/games/$platform" params={{ platform: p }}>
                  <Badge
                    variant={p === platform ? "default" : "secondary"}
                    className="cursor-pointer hover:bg-secondary/80 text-xs"
                  >
                    {p}
                  </Badge>
                </Link>
              ))}
            </div>
          </div>
        </div>
      </header>

      <ScrollArea className="h-[calc(100vh-10.5rem)]">
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 p-4">
          {games?.map((game) => (
            <GameCard key={game.id} game={game} />
          ))}
        </div>
      </ScrollArea>
    </div>
  )
}
