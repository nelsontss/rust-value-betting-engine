import { createRootRoute, Link, Outlet } from "@tanstack/react-router"
import { useAlertsSubscription } from "@/hooks/useAlerts"
import { useAlertsStore } from "@/stores/alerts"

export const rootRoute = createRootRoute({
  component: RootLayout,
})

function RootLayout() {
  useAlertsSubscription()
  const count = useAlertsStore((s) => s.alerts.length)
  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-20 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center gap-1 px-4 h-14">
          <Link
            to="/"
            activeOptions={{ exact: true }}
            activeProps={{ className: "bg-secondary text-secondary-foreground" }}
            className="rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          >
            Dashboard
          </Link>
          <Link
            to="/games"
            activeOptions={{ exact: false }}
            activeProps={{ className: "bg-secondary text-secondary-foreground" }}
            className="rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          >
            Games
          </Link>
          <Link
            to="/statistics"
            activeOptions={{ exact: false }}
            activeProps={{ className: "bg-secondary text-secondary-foreground" }}
            className="rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          >
            Statistics
          </Link>
          <Link
            to="/alerts"
            activeProps={{ className: "bg-secondary text-secondary-foreground" }}
            className="rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          >
            Alerts {count > 0 && <span className="ml-1 bg-destructive text-destructive-foreground rounded-full px-1.5 py-0.5 text-xs">{count}</span>}
          </Link>
        </div>
      </header>
      <Outlet />
      {/* <AlertsToaster /> disabled temporarily */}
    </div>
  )
}