import { createRoute } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { GamesPage } from "@/components/clusters/GamesPage"

export const gamesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/games",
  component: GamesPage,
})
