// Tab data for 035 — deliberately NO `id` field: the auto-:key heuristic
// ('Tag-N-' + (item?.id ?? item)) would collapse every tab to the same
// constant key ('EditorTab-1-[object Object]'), making Vue reuse a single
// component instance across iterations. The explicit `key:` prop on the
// instantiation is the fix.
export interface TabInfo {
    path: string
    label: string
}

export function initialTabs(): TabInfo[] {
    return [
        { path: '/notes/jade.md', label: 'jade' },
        { path: '/notes/garden.md', label: 'garden' },
        { path: '/notes/autodown.md', label: 'autodown' },
    ]
}
