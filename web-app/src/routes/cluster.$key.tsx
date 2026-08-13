import { createRoute, useNavigate, useParams } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { useCluster } from "@/hooks/useClusters"
import { ClusterDetail } from "@/components/clusters/ClusterDetail"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ArrowLeft } from "lucide-react"

export const clusterRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/cluster/$key",
  component: ClusterPage,
})

function ClusterPage() {
  const { key } = useParams({ from: clusterRoute.id })
  const { data: cluster, isLoading, error } = useCluster(key)
  const navigate = useNavigate()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-muted-foreground">
        Loading cluster...
      </div>
    )
  }

  if (error || !cluster) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-destructive">
        Failed to load cluster
      </div>
    )
  }

  const rep = cluster.representative_game
  const platforms = [...new Set(cluster.games.map((g) => g.platform))]

  return (
    <div className="flex flex-col h-[calc(100vh-3.5rem)]">
      <header className="sticky top-14 z-10 flex items-center gap-3 border-b bg-background/95 backdrop-blur px-4 py-2 shrink-0">
        <Button variant="ghost" size="sm" onClick={() => navigate({ to: "/" })}>
          <ArrowLeft className="size-4 mr-1" />
          Back
        </Button>
        {rep && (
          <div className="flex items-center gap-3 min-w-0 flex-1">
            <div className="min-w-0">
              <span className="text-sm font-semibold truncate block">
                {rep.home_team} vs {rep.away_team}
              </span>
              <span className="text-xs text-muted-foreground">
                {rep.competition} &middot; {rep.country} &middot;{" "}
                {new Date(rep.date).toLocaleDateString()}
              </span>
            </div>
            <div className="flex items-center gap-1 shrink-0 ml-auto">
              {platforms.map((p) => (
                <Badge key={p} variant="secondary" className="text-xs">
                  {p}
                </Badge>
              ))}
            </div>
          </div>
        )}
      </header>

      <div className="flex-1 overflow-hidden">
        <ClusterDetail cluster={cluster} />
      </div>
    </div>
  )
}
