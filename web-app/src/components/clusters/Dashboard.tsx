import { useClusters, useClusterSubscription } from "@/hooks/useClusters"
import { useClustersUIStore } from "@/stores/clusters-ui"
import { ClusterGrid } from "./ClusterGrid"
import { ClusterDetail } from "./ClusterDetail"
import { useCluster } from "@/hooks/useClusters"
import { X } from "lucide-react"
import { Button } from "@/components/ui/button"

export function Dashboard() {
  const { data: clusters, isLoading, error } = useClusters()
  const selectedClusterId = useClustersUIStore((s) => s.selectedClusterId)
  const setSelectedClusterId = useClustersUIStore((s) => s.setSelectedClusterId)
  const { data: selectedCluster } = useCluster(selectedClusterId)

  useClusterSubscription()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-muted-foreground">
        Loading clusters...
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)] text-destructive">
        Failed to load clusters: {(error as Error).message}
      </div>
    )
  }

  return (
    <>
      <header className="sticky top-14 z-10 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center justify-between px-4 h-14">
          <h1 className="text-lg font-semibold">Clusters Dashboard</h1>
          <span className="text-sm text-muted-foreground">
            {clusters?.length ?? 0} clusters
          </span>
        </div>
      </header>

      {selectedCluster ? (
        <div className="relative">
          <div className="sticky top-28 z-10 flex items-center gap-2 border-b bg-background/95 backdrop-blur px-4 py-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setSelectedClusterId(null)}
            >
              <X className="size-4 mr-1" />
              Back
            </Button>
            <span className="text-sm font-medium">
              Cluster detail
            </span>
          </div>
          <ClusterDetail cluster={selectedCluster} />
        </div>
      ) : (
        <ClusterGrid clusters={clusters ?? []} />
      )}
    </>
  )
}
