import { Link } from "@tanstack/react-router"
import { useGames, usePlatforms } from "@/hooks/useGames"
import { GameCard } from "./GameCard"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Badge } from "@/components/ui/badge"

export function GamesPage() {
  const { data: games, isLoading, error } = useGames()
  const { data: platforms } = usePlatforms()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-muted-foreground">
        Loading games...
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
          <h1 className="text-lg font-semibold">All Games</h1>
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">
              {games?.length ?? 0} games
            </span>
            <div className="flex gap-1 ml-2">
              {platforms.map((platform) => (
                <Link key={platform} to="/games/$platform" params={{ platform }}>
                  <Badge variant="secondary" className="cursor-pointer hover:bg-secondary/80 text-xs">
                    {platform}
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
