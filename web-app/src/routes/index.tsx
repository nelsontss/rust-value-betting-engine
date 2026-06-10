import { createRoute } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { Dashboard } from "@/components/clusters/Dashboard"

export const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: Dashboard,
})
