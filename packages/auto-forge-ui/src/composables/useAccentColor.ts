import { readonly, ref, onMounted } from 'vue'

/* ═════════════════════════════════════════════════════════════════════════════
   useAccentColor
   ────────────────────────────────────────────────────────────────────────────
   Lets the user pick a primary accent color.  The curated palette is drawn
   from the website's design language (website/.vitepress/theme/style.css and
   hero components) so the Forge app feels like a sibling product.
   ═════════════════════════════════════════════════════════════════════════════ */

const STORAGE_KEY = 'autoforge-accent-color'

export type AccentName = 'indigo' | 'coral' | 'ocean' | 'sage' | 'amber'

interface AccentPalette {
  name: AccentName
  label: string
  /* VitePress-compatible brand tokens */
  brand1: string
  brand2: string
  brand3: string
  brandSoft: string
  /* shadcn primary HSL triplet */
  primaryHsl: string
}

/*
 * Curated colours taken from / inspired by website/.vitepress/theme/
 *
 * Indigo  – the website's main brand (#6366f1 → #818cf8 → #4f46e5)
 * Coral   – a warm, energetic rose-coral that pairs well with indigo
 * Ocean   – the blue used in website gradients (#3b82f6)
 * Sage    – a fresh, modern green (not in the website, but balances the set)
 * Amber   – the warm gold from AIHero.vue (#f59e0b)
 */
const PALETTES: Record<AccentName, AccentPalette> = {
  indigo: {
    name: 'indigo',
    label: 'Indigo',
    brand1: '#6366f1',
    brand2: '#818cf8',
    brand3: '#4f46e5',
    brandSoft: 'rgba(99, 102, 241, 0.14)',
    primaryHsl: '239 84% 67%',
  },
  coral: {
    name: 'coral',
    label: 'Coral',
    brand1: '#e85d75',
    brand2: '#f08090',
    brand3: '#c9445e',
    brandSoft: 'rgba(232, 93, 117, 0.14)',
    primaryHsl: '350 75% 64%',
  },
  ocean: {
    name: 'ocean',
    label: 'Ocean',
    brand1: '#3b82f6',
    brand2: '#60a5fa',
    brand3: '#2563eb',
    brandSoft: 'rgba(59, 130, 246, 0.14)',
    primaryHsl: '217 91% 60%',
  },
  sage: {
    name: 'sage',
    label: 'Sage',
    brand1: '#10b981',
    brand2: '#34d399',
    brand3: '#059669',
    brandSoft: 'rgba(16, 185, 129, 0.14)',
    primaryHsl: '160 84% 39%',
  },
  amber: {
    name: 'amber',
    label: 'Amber',
    brand1: '#f59e0b',
    brand2: '#fbbf24',
    brand3: '#d97706',
    brandSoft: 'rgba(245, 158, 11, 0.14)',
    primaryHsl: '38 92% 50%',
  },
}

export const ACCENT_OPTIONS = Object.values(PALETTES)

const _current = ref<AccentName>('indigo')

function apply(name: AccentName) {
  const p = PALETTES[name]
  const root = document.documentElement
  root.style.setProperty('--vp-c-brand-1', p.brand1)
  root.style.setProperty('--vp-c-brand-2', p.brand2)
  root.style.setProperty('--vp-c-brand-3', p.brand3)
  root.style.setProperty('--vp-c-brand-soft', p.brandSoft)
  root.style.setProperty('--primary', p.primaryHsl)
}

export function useAccentColor() {
  const current = readonly(_current)

  function setAccent(name: AccentName) {
    _current.value = name
    localStorage.setItem(STORAGE_KEY, name)
    apply(name)
  }

  onMounted(() => {
    const stored = localStorage.getItem(STORAGE_KEY) as AccentName | null
    const initial: AccentName = stored && PALETTES[stored] ? stored : 'indigo'
    _current.value = initial
    apply(initial)
  })

  return {
    current,
    setAccent,
    options: ACCENT_OPTIONS,
  }
}
