import type { Cluster } from "@/types/cluster"
import { ClusterCard } from "./ClusterCard"
import { ScrollArea } from "@/components/ui/scroll-area"

interface ClusterGridProps {
  clusters: Cluster[]
}

export function ClusterGrid({ clusters }: ClusterGridProps) {
  return (
    <ScrollArea className="h-[calc(100vh-8rem)]">
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 p-4">
        {clusters.map((cluster) => (
          <ClusterCard key={cluster.id} cluster={cluster} />
        ))}
      </div>
    </ScrollArea>
  )
}
