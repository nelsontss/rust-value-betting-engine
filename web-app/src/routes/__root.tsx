import { createRootRoute, Link, Outlet } from "@tanstack/react-router"
import { Button } from "@/components/ui/button"

export const rootRoute = createRootRoute({
  component: RootLayout,
})

function RootLayout() {
  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-10 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center justify-between px-4 h-14">
          <div className="flex items-center gap-2">
            <Link to="/">
              <Button variant="ghost" size="sm">
                Clusters
              </Button>
            </Link>
            <Link to="/games">
              <Button variant="ghost" size="sm">
                Games
              </Button>
            </Link>
          </div>
        </div>
      </header>
      <Outlet />
    </div>
  )
}
