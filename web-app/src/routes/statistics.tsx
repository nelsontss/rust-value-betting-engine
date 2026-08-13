import { createRoute } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { StatisticsPage } from "@/components/statistics/StatisticsPage"

export const statisticsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/statistics",
  component: StatisticsPage,
})