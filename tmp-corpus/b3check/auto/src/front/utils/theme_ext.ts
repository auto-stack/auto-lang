// theme_ext.ts — hand-written TS extension for sidebar.at: re-exports the
// generated Theme store composable (dual-resolution shim, jade pattern) and
// the accent palette for the swatch loop. The palette/DOM/localStorage logic
// itself lives in src/lib/api.ts (stores can only import via back.api).
//
// reactive() wrap: the generated composable returns a PLAIN object of refs;
// nested template access (themeStore.current) does not unwrap without it.
import { reactive } from 'vue'
import { useThemeStore as useGeneratedThemeStore } from '../../../../src/stores/auto/useThemeStore'

export function useThemeStore(): any {
  return reactive(useGeneratedThemeStore())
}

export { ACCENT_OPTIONS } from '../../../../src/lib/api'
