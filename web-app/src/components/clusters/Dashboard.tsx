import { useClusters, useClusterSubscription } from "@/hooks/useClusters"
import { ClusterGrid } from "./ClusterGrid"

export function Dashboard() {
  const { data: clusters, isLoading, error } = useClusters()

  useClusterSubscription()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen text-muted-foreground">
        Loading clusters...
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-screen text-destructive">
        Failed to load clusters: {(error as Error).message}
      </div>
    )
  }

  return (
    <>
      <header className="sticky top-0 z-10 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center justify-between px-4 h-14">
          <h1 className="text-lg font-semibold">Clusters Dashboard</h1>
          <span className="text-sm text-muted-foreground">
            {clusters?.length ?? 0} clusters
          </span>
        </div>
      </header>

      <ClusterGrid clusters={clusters ?? []} />
    </>
  )
}
