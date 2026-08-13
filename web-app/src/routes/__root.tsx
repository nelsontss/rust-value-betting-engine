import { createRootRoute, Link, Outlet } from "@tanstack/react-router"

export const rootRoute = createRootRoute({
  component: RootLayout,
})

function RootLayout() {
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
        </div>
      </header>
      <Outlet />
    </div>
  )
}