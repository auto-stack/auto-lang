// controls_ext.ts — hand-written TS extension for the Phase 4 control
// widgets (ScalarFields/TableField/ConfigEditor): relays the transport-layer
// helpers (use-block imports must live under auto/src/front/).
export {
  loadEnum,
  enumUrlOf,
  tableInfo,
  setCell,
  blankRow,
  mergeCols,
  loadColumnOptions,
  configEntries,
  setCfgEntry,
  cfgProvider,
  entriesBody,
  confirmSaveOnce,
  fetchConfigSafe,
  putConfigSafe,
  deleteBlockSafe,
} from '../../../../src/lib/api'
export { default as ScalarFields } from '../../../../src/components/ScalarFields.vue'
export { default as TableField } from '../../../../src/components/TableField.vue'
export { addBlockBody, bodyHas, confirmDeleteBlock, removeRowAt } from '../../../../src/lib/api'
