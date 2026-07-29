// Hand-written Vue composable consumed by `use { composable: ... }`.
// The generated SFC calls it once at <script setup> top level:
//   const clock = useClock()
// and `on` handlers reach it as `clock.stamp()`.

export function useClock() {
  const stamp = (): string => new Date().toLocaleTimeString()
  return { stamp }
}
