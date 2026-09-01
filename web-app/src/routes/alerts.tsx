import { createRoute } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { AlertsPage } from "@/components/alerts/AlertsPage"

export const alertsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/alerts",
  component: AlertsPage,
})
