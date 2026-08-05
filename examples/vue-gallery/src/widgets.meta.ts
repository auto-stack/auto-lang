// Human-maintained widget metadata (blurb + group), keyed by widget name.
// The name+route list is in widgets.generated.ts (machine-generated).

export interface WidgetMeta {
  group: string
  blurb: string
}

export const widgetMeta: Record<string, WidgetMeta> = {
  button: { group: 'Form', blurb: 'Action trigger with variants & sizes' },
  input: { group: 'Form', blurb: 'Single-line text field' },
  textarea: { group: 'Form', blurb: 'Multi-line text field' },
  checkbox: { group: 'Form', blurb: 'Boolean selector' },
  switch: { group: 'Form', blurb: 'Toggle control' },
  label: { group: 'Form', blurb: 'Form field caption' },
  card: { group: 'Layout', blurb: 'Container with header / content / footer' },
  separator: { group: 'Layout', blurb: 'Horizontal / vertical divider' },
  badge: { group: 'Feedback', blurb: 'Compact status label' },
  avatar: { group: 'Feedback', blurb: 'User image with fallback' },
  dialog: { group: 'Overlay / Nav', blurb: 'Modal overlay' },
  tabs: { group: 'Overlay / Nav', blurb: 'Switchable panels' },
}
