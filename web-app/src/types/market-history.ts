export interface MarketOdd {
  value: number
}

export type MarketData =
  | {
      type: "MatchResult"
      home: MarketOdd
      draw: MarketOdd
      away: MarketOdd
    }
  | {
      type: "Moneyline"
      home: MarketOdd
      away: MarketOdd
    }
  | {
      type: "DoubleChance"
      home_or_draw: MarketOdd
      home_or_away: MarketOdd
      draw_or_away: MarketOdd
    }
  | {
      type: "Total"
      line: number
      over: MarketOdd
      under: MarketOdd
    }
  | {
      type: "Handicap"
      line: number
      home: MarketOdd
      draw: MarketOdd
      away: MarketOdd
    }
  | {
      type: "AsianHandicap"
      line: number
      home: MarketOdd
      away: MarketOdd
    }

export interface MarketDataPoint {
  timestamp: string
  market: MarketData
}

export interface MarketHistoryResponse {
  game_id: string
  markets: MarketDataPoint[]
}

export interface MarketDataPointResponse {
  game_id: string
  timestamp: string
  market: MarketData
}
