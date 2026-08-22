// useTick.ts — k4 hand-written composable (named export). Forwarded to
// callers through the port module, never imported directly.
import { ref } from 'vue'

export function useTick() {
  const count = ref(0)
  const label = () => `tick ${count.value}`
  return { count, label }
}
