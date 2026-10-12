import screenshotVariants from './screenshot-variants.json'

const variants: Record<string, string> = screenshotVariants

/** Local delivery image; callers retain the capture path for explicit original downloads. */
export function screenshotSrc(source: string): string {
  return variants[source] || source
}
