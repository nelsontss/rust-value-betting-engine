import { create } from "zustand"

export type ChangeDirection = "up" | "down" | "mixed" | null

interface ClustersUIState {
  selectedClusterId: string | null
  setSelectedClusterId: (id: string | null) => void
  clusterChanges: Record<string, ChangeDirection>
  setClusterChange: (id: string, direction: ChangeDirection) => void
}

export const useClustersUIStore = create<ClustersUIState>((set) => ({
  selectedClusterId: null,
  setSelectedClusterId: (id) =>
    set((state) => ({
      selectedClusterId: id,
      clusterChanges: id
        ? { ...state.clusterChanges, [id]: null }
        : state.clusterChanges,
    })),
  clusterChanges: {},
  setClusterChange: (id, direction) =>
    set((state) => ({
      clusterChanges: { ...state.clusterChanges, [id]: direction },
    })),
}))
