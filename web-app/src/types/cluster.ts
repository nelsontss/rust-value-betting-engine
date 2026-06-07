export interface Cluster {
  id: string
  games: Game[]
  representative_game: Game | null
  updated_at: string
}

export interface Game {
  id: string
  home_team: string
  away_team: string
  country: string
  competition: string
  platform: string
  date: string
  markets: Market[]
}

export type Market =
  | { type: "MatchResult"; home: Odd; draw: Odd; away: Odd }
  | { type: "Moneyline"; home: Odd; away: Odd }
  | { type: "Total"; line: number; over: Odd; under: Odd }
  | { type: "Handicap"; line: number; home: Odd; draw: Odd; away: Odd }
  | { type: "AsianHandicap"; line: number; home: Odd; away: Odd }

export interface Odd {
  value: number
}
