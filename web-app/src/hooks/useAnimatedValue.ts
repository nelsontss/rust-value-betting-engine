import { useRef, useState, useEffect } from "react"
import { approxEq } from "@/lib/odds"

export function useChangeDirection(current: number): "up" | "down" | null {
  const prevRef = useRef(current)
  const [direction, setDirection] = useState<"up" | "down" | null>(null)

  useEffect(() => {
    if (!approxEq(prevRef.current, current)) {
      setDirection(current > prevRef.current ? "up" : "down")
      prevRef.current = current
      const timer = setTimeout(() => setDirection(null), 800)
      return () => clearTimeout(timer)
    }
  }, [current])

  return direction
}
