import { useClusters, useClusterSubscription } from "@/hooks/useClusters"
import { ClusterTable } from "./ClusterTable"
import { ClusterInspector } from "./ClusterInspector"

interface DashboardProps {
  clusterId: string
  marketKey: string
  onSelectCluster: (id: string) => void
  onSelectMarket: (key: string) => void
}

export function Dashboard({
  clusterId,
  marketKey,
  onSelectCluster,
  onSelectMarket,
}: DashboardProps) {
  const { data: clusters, isLoading, error } = useClusters()

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

  const list = clusters ?? []
  const selected = list.find((c) => c.id === clusterId) ?? list[0] ?? null

  return (
    <div className="flex flex-col h-[calc(100vh-3.5rem)]">
      <header className="sticky top-14 z-10 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex items-center justify-between px-4 h-14">
          <h1 className="text-lg font-semibold">Clusters Dashboard</h1>
          <span className="text-sm text-muted-foreground">
            {list.length} clusters
          </span>
        </div>
      </header>

      <div className="flex flex-1 min-h-0">
        <aside className="flex-1 min-w-0 border-r min-h-0">
          <ClusterTable
            clusters={list}
            selectedId={selected?.id ?? null}
            onSelect={onSelectCluster}
          />
        </aside>

        <main className="flex-1 min-w-0 overflow-y-auto">
          {selected ? (
            <ClusterInspector
              key={selected.id}
              cluster={selected}
              selectedGroupKey={marketKey || null}
              onSelectGroupKey={onSelectMarket}
            />
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              No clusters available
            </div>
          )}
        </main>
      </div>
    </div>
  )
}
