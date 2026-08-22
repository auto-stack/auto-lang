// label_fmt.ts — k4 hand-written pure fn (named export). Bound by the port
// module and wrapped, exercising the fn-kind (wrapper) path alongside the
// re-export paths.
export function relLabel(n: number): string {
  return n === 1 ? `${n} item` : `${n} items`
}
