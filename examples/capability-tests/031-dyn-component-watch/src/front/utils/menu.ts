// Hand-written TS "extension" for 031: menu items whose `icon` field is a
// Vue component (lucide), plus the CustomScrollbar-style thumb geometry
// helper that reads the DOM imperatively.
import { FilePlus2, Copy, Trash2 } from 'lucide-vue-next'
import type { Component } from 'vue'

export interface MenuItem {
    id: string
    label: string
    keywords: string
    icon: Component
}

const ITEMS: MenuItem[] = [
    { id: 'new', label: 'New Note', keywords: 'new create note', icon: FilePlus2 },
    { id: 'duplicate', label: 'Duplicate Note', keywords: 'duplicate copy note', icon: Copy },
    { id: 'delete', label: 'Delete Note', keywords: 'delete remove trash note', icon: Trash2 },
]

export function menuItems(): MenuItem[] {
    return ITEMS
}

export function filterItems(query: string): MenuItem[] {
    const q = query.trim().toLowerCase()
    if (q === '') return ITEMS
    return ITEMS.filter(
        (it) => it.keywords.includes(q) || it.label.toLowerCase().includes(q),
    )
}

// CustomScrollbar thumb sizing: read the track's box from the DOM and derive
// the thumb height from the ratio prop. Returns a px height.
export function thumbHeight(track: HTMLElement | null, ratio: number): number {
    if (!track) return 10
    const clamped = Math.min(Math.max(ratio, 0), 100)
    return Math.max(Math.round((track.clientHeight * clamped) / 100), 10)
}
