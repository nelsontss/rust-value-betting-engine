import { createRouter } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { indexRoute } from "./index"
import { gamesRoute } from "./games"
import { platformGamesRoute } from "./games.$platform"
import { clusterRoute } from "./cluster.$key"

const routeTree = rootRoute.addChildren([
  indexRoute,
  gamesRoute,
  platformGamesRoute,
  clusterRoute,
])

export const router = createRouter({ routeTree })

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router
  }
}
