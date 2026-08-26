// modules_store_ext.ts — dual-resolution shim for the Modules store (jade
// pattern): generated components/stores import through this module so both
// the gen tree and the host tree resolve. Also carries the sidebar's search
// filtering (value-shape math stays in TS, per the ext policy).
//
// reactive() wrap: the generated composable returns a PLAIN object of refs;
// nested template access (modulesStore.expanded) does not unwrap without it.
import { reactive } from 'vue'
import { useModulesStore as useGeneratedModulesStore } from '../../../../src/stores/auto/useModulesStore'

export function useModulesStore(): any {
  return reactive(useGeneratedModulesStore())
}

/** Sidebar search filter over the standalone list (name/description match). */
export function filterStandalone(list: any[], q: string): any[] {
  const s = q.trim().toLowerCase()
  if (!s) return list
  return list.filter(
    (m) => m.name.toLowerCase().includes(s) || m.description.toLowerCase().includes(s),
  )
}

/** Sidebar search filter over groups; members filtered, empty groups dropped. */
export function filterGroups(groups: any[], q: string): any[] {
  const s = q.trim().toLowerCase()
  if (!s) return groups
  return groups
    .map((g) => ({
      ...g,
      members: g.members.filter(
        (m: any) =>
          m.name.toLowerCase().includes(s) || m.description.toLowerCase().includes(s),
      ),
    }))
    .filter((g) => g.members.length > 0)
}
