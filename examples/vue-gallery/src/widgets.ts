// Catalog of v1 widgets grouped for the sidebar + Home overview.
// Each entry maps a route to its showcase page.
export interface WidgetEntry {
  name: string
  route: string
  blurb: string
}

export interface WidgetGroup {
  label: string
  widgets: WidgetEntry[]
}

export const widgetGroups: WidgetGroup[] = [
  {
    label: 'Form',
    widgets: [
      { name: 'button', route: '/button', blurb: 'Action trigger with variants & sizes' },
      { name: 'input', route: '/input', blurb: 'Single-line text field' },
      { name: 'textarea', route: '/textarea', blurb: 'Multi-line text field' },
      { name: 'checkbox', route: '/checkbox', blurb: 'Boolean selector' },
      { name: 'switch', route: '/switch', blurb: 'Toggle control' },
      { name: 'label', route: '/label', blurb: 'Form field caption' },
    ],
  },
  {
    label: 'Layout',
    widgets: [
      { name: 'card', route: '/card', blurb: 'Container with header / content / footer' },
      { name: 'separator', route: '/separator', blurb: 'Horizontal / vertical divider' },
    ],
  },
  {
    label: 'Feedback',
    widgets: [
      { name: 'badge', route: '/badge', blurb: 'Compact status label' },
      { name: 'avatar', route: '/avatar', blurb: 'User image with fallback' },
    ],
  },
  {
    label: 'Overlay / Nav',
    widgets: [
      { name: 'dialog', route: '/dialog', blurb: 'Modal overlay' },
      { name: 'tabs', route: '/tabs', blurb: 'Switchable panels' },
    ],
  },
]
