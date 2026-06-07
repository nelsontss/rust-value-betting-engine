import { useEffect } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import type { Cluster } from "@/types/cluster"
import { apiUrl, sseUrl } from "@/lib/api"
import { detectClusterChange } from "@/lib/odds"
import { useClustersUIStore } from "@/stores/clusters-ui"

const CLUSTERS_KEY = ["clusters"]

async function fetchClusters(): Promise<Cluster[]> {
  const res = await fetch(apiUrl("/clusters"))
  if (!res.ok) throw new Error(`Failed to fetch clusters: ${res.status}`)
  return res.json()
}

export function useClusters() {
  return useQuery({
    queryKey: CLUSTERS_KEY,
    queryFn: fetchClusters,
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  })
}

export function useCluster(id: string | null) {
  return useQuery({
    queryKey: ["cluster", id],
    queryFn: async () => {
      const res = await fetch(apiUrl(`/clusters/${id}`))
      if (!res.ok) throw new Error(`Failed to fetch cluster: ${res.status}`)
      return res.json() as Promise<Cluster>
    },
    enabled: !!id,
  })
}

export function useClusterSubscription() {
  const queryClient = useQueryClient()
  const setClusterChange = useClustersUIStore((s) => s.setClusterChange)

  useEffect(() => {
    const url = sseUrl("/sse/clusters")
    let es: EventSource | null = null
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null
    let reconnectAttempts = 0
    const maxReconnectAttempts = 10
    const baseDelay = 1000

    function connect() {
      es = new EventSource(url)

      es.onmessage = (event) => {
        reconnectAttempts = 0
        try {
          const cluster: Cluster = JSON.parse(event.data)
          queryClient.setQueryData<Cluster[]>(CLUSTERS_KEY, (prev) => {
            if (!prev) return [cluster]
            const idx = prev.findIndex((c) => c.id === cluster.id)
            if (idx >= 0) {
              const prevCluster = prev[idx]
              const direction = detectClusterChange(prevCluster, cluster)
              if (direction) setClusterChange(cluster.id, direction)

              const next = [...prev]
              next[idx] = cluster
              return next
            }
            return [cluster, ...prev]
          })
          queryClient.setQueryData(["cluster", cluster.id], cluster)
        } catch {
          // skip malformed events
        }
      }

      es.onerror = () => {
        es?.close()
        scheduleReconnect()
      }
    }

    function scheduleReconnect() {
      if (reconnectAttempts >= maxReconnectAttempts) return
      const delay = baseDelay * Math.pow(2, reconnectAttempts)
      reconnectAttempts++
      reconnectTimer = setTimeout(connect, delay)
    }

    connect()

    return () => {
      es?.close()
      if (reconnectTimer) clearTimeout(reconnectTimer)
    }
  }, [queryClient, setClusterChange])
}

