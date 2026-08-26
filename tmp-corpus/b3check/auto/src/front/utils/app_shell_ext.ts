// app_shell_ext.ts — AppShell dispatches to the content components.
// Sidebar/DaemonView/CollectionBrowser are generated widgets (direct
// references); ConfigEditor stays hand-written until Phase 4 and is consumed
// through this re-export (jade gen_components stub pattern, in reverse).
import ConfigEditor from '../../../../src/components/ConfigEditor.vue'

export { ConfigEditor }
