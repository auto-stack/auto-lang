// collection_store_ext.ts — dual-resolution shim for the Collection store +
// the (still hand-written until Plan 006 Phase 4) leaf controls the browser
// consumes. reactive() wrap: the generated composable returns plain refs.
import { reactive } from 'vue'
import { useCollectionStore as useGeneratedCollectionStore } from '../../../../src/stores/auto/useCollectionStore'

export function useCollectionStore(): any {
  return reactive(useGeneratedCollectionStore())
}

export { filterEntities } from '../../../../src/lib/api'
export { default as ScalarFields } from '../../../../src/components/ScalarFields.vue'
export { default as TableField } from '../../../../src/components/TableField.vue'
