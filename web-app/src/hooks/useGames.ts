import { useQuery } from "@tanstack/react-query"
import type { Game } from "@/types/cluster"
import { apiUrl } from "@/lib/api"

const GAMES_KEY = ["games"]
const PLATFORMS_KEY = ["platforms"]

async function fetchGames(): Promise<Game[]> {
  const res = await fetch(apiUrl("/games"))
  if (!res.ok) throw new Error(`Failed to fetch games: ${res.status}`)
  return res.json()
}

async function fetchPlatformGames(platform: string): Promise<Game[]> {
  const res = await fetch(apiUrl(`/games/${platform}`))
  if (!res.ok) throw new Error(`Failed to fetch games for ${platform}: ${res.status}`)
  return res.json()
}

async function fetchPlatforms(): Promise<string[]> {
  const res = await fetch(apiUrl("/platforms"))
  if (!res.ok) throw new Error(`Failed to fetch platforms: ${res.status}`)
  return res.json()
}

export function useGames() {
  return useQuery({
    queryKey: GAMES_KEY,
    queryFn: fetchGames,
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  })
}

export function usePlatformGames(platform: string) {
  return useQuery({
    queryKey: ["games", platform],
    queryFn: () => fetchPlatformGames(platform),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  })
}

export function usePlatforms() {
  return useQuery({
    queryKey: PLATFORMS_KEY,
    queryFn: fetchPlatforms,
    staleTime: 300_000,
    refetchOnWindowFocus: false,
  })
}
