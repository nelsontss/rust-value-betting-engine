import { createRoute } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { PlatformGamesPage } from "@/components/clusters/PlatformGamesPage"

export const platformGamesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/games/$platform",
  component: PlatformGamesPage,
})
