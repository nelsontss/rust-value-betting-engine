import { createRoute } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { MarketHistoryPage } from "@/components/clusters/MarketHistoryPage"

export const marketHistoryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/market-history/$gameId",
  component: MarketHistoryPage,
})
