import { useChangeDirection } from "@/hooks/useAnimatedValue"
import type { Odd } from "@/types/cluster"

interface OddsCellProps {
  label: string
  odd: Odd
}

export function OddsCell({ label, odd }: OddsCellProps) {
  const direction = useChangeDirection(odd.value)

  return (
    <div className="flex flex-col items-center gap-0.5">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span
        className={`font-mono text-sm font-medium tabular-nums rounded px-1.5 py-0.5 transition-colors ${
          direction === "up" ? "animate-flash-up" : ""
        } ${direction === "down" ? "animate-flash-down" : ""}`}
      >
        {odd.value.toFixed(2)}
      </span>
    </div>
  )
}
