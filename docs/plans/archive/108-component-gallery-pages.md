# Component Gallery Page Files Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Update 7 existing crude widget pages and add 38 missing widget .at page files for the component gallery, following the established pattern from button.at and input.at.

**Architecture:** Each widget page follows a consistent structure: Widget definition with model/view, h1 title with description, Installation section with bash codeblock, Simple/Examples sections with preview-card components, and Properties table. Content is sourced from shadcn-vue.com documentation.

**Tech Stack:** AutoLang AURA widgets, shadcn-vue components, Vue Router, Prism.js for syntax highlighting

---

## Summary

**Existing pages to update (7):** accordion, badge, card, checkbox, label, tabs, text (crude implementations missing Properties tables and examples)

**Complete pages (3):** button, input, index (no changes needed)

**Missing pages to create (38):** alert, alertdialog, avatar, breadcrumb, calendar, carousel, collapsible, combobox, command, contextmenu, datatable, datepicker, dialog, drawer, dropdownmenu, form, hovercard, menubar, navigationmenu, pagination, popover, progress, radiogroup, scrollarea, select, separator, sheet, sidebar, skeleton, slider, sonner, switch, table, textarea, toast, toggle, togglegroup, tooltip

**Page Template Structure:**
```auto
// pages/{widget}.at - {Widget} component documentation page

widget {Widget}Page {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "{Widget}") {}
            text (text: "{Description from shadcn-vue}") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add {widget}") {}

            h2 (text: "Simple") {}
            preview-card (id: "{widget}-basic") {
                // Basic usage example
            }

            h2 (text: "Examples") {}
            // Additional examples with h3 subsections

            h2 (text: "Properties") {}
            table {
                // Property definitions
            }
        }
    }
}
```

---

## Batch 0: Update Existing Crude Pages

### Task 0.1: Update Badge Page

**Files:**
- Modify: `examples/component-gallery/source/front/pages/badge.at`

**Reference:** https://www.shadcn-vue.com/docs/components/badge

**Current State:** Very basic, missing variants and Properties table.

**Step 1: Update badge.at with complete content**

```auto
// pages/badge.at - Badge component documentation page

widget BadgePage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Badge") {}
            text (text: "Displays a badge or a label that draws attention and labels items.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add badge") {}

            h2 (text: "Simple") {}
            preview-card (id: "badge-basic") {
                badge (text: "Badge") {}
            }

            h2 (text: "Variants") {}
            preview-card (id: "badge-variants") {
                row (gap: "2") {
                    badge (text: "Default") {}
                    badge (text: "Secondary", variant: "secondary") {}
                    badge (text: "Destructive", variant: "destructive") {}
                    badge (text: "Outline", variant: "outline") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "text") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Badge text content (slot)") {}
                    }
                    tr {
                        td (text: "variant") {}
                        td (text: "string") {}
                        td (text: "default") {}
                        td (text: "default, secondary, destructive, outline") {}
                        td (text: "Badge style variant") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/badge.at
git commit -m "feat(docs): improve badge component page with variants and properties"
```

---

### Task 0.2: Update Checkbox Page

**Files:**
- Modify: `examples/component-gallery/source/front/pages/checkbox.at`

**Reference:** https://www.shadcn-vue.com/docs/components/checkbox

**Current State:** Very basic, missing examples with label and Properties table.

**Step 1: Update checkbox.at with complete content**

```auto
// pages/checkbox.at - Checkbox component documentation page

widget CheckboxPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Checkbox") {}
            text (text: "A control that allows the user to toggle between checked and not checked.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add checkbox") {}

            h2 (text: "Simple") {}
            preview-card (id: "checkbox-basic") {
                row (gap: "2", class: "items-center") {
                    checkbox (id: "terms1") {}
                    label (text: "Accept terms and conditions", for: "terms1") {}
                }
            }

            h2 (text: "Disabled") {}
            preview-card (id: "checkbox-disabled") {
                row (gap: "2", class: "items-center") {
                    checkbox (id: "terms2", disabled: "true") {}
                    label (text: "Accept terms and conditions", for: "terms2") {}
                }
            }

            h2 (text: "Checked") {}
            preview-card (id: "checkbox-checked") {
                row (gap: "2", class: "items-center") {
                    checkbox (id: "terms3", checked: "true") {}
                    label (text: "Accept terms and conditions", for: "terms3") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "checked") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Checkbox state") {}
                    }
                    tr {
                        td (text: "disabled") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Disables the checkbox") {}
                    }
                    tr {
                        td (text: "id") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "ID for label association") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/checkbox.at
git commit -m "feat(docs): improve checkbox component page with examples and properties"
```

---

### Task 0.3: Update Accordion Page

**Files:**
- Modify: `examples/component-gallery/source/front/pages/accordion.at`

**Reference:** https://www.shadcn-vue.com/docs/components/accordion

**Current State:** Just a placeholder with no actual accordion demo.

**Step 1: Update accordion.at with complete content**

```auto
// pages/accordion.at - Accordion component documentation page

widget AccordionPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Accordion") {}
            text (text: "A vertically stacked set of interactive headings that each reveal a section of content.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add accordion") {}

            h2 (text: "Simple") {}
            preview-card (id: "accordion-basic") {
                accordion (type: "single", collapsible: "true", class: "w-full") {
                    accordion-item (value: "item-1") {
                        accordion-trigger (text: "Is it accessible?") {}
                        accordion-content {
                            text (text: "Yes. It adheres to the WAI-ARIA design pattern.") {}
                        }
                    }
                    accordion-item (value: "item-2") {
                        accordion-trigger (text: "Is it styled?") {}
                        accordion-content {
                            text (text: "Yes. It comes with default styles that match the other components.") {}
                        }
                    }
                    accordion-item (value: "item-3") {
                        accordion-trigger (text: "Is it animated?") {}
                        accordion-content {
                            text (text: "Yes. It's animated by default, but you can disable it if you prefer.") {}
                        }
                    }
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "type") {}
                        td (text: "string") {}
                        td (text: "single") {}
                        td (text: "single, multiple") {}
                        td (text: "Single or multiple items open") {}
                    }
                    tr {
                        td (text: "collapsible") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Allow all items to be collapsed") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/accordion.at
git commit -m "feat(docs): improve accordion component page with working demo and properties"
```

---

### Task 0.4: Update Label Page

**Files:**
- Modify: `examples/component-gallery/source/front/pages/label.at`

**Reference:** https://www.shadcn-vue.com/docs/components/label

**Current State:** Basic, missing example with input association and Properties table.

**Step 1: Update label.at with complete content**

```auto
// pages/label.at - Label component documentation page

widget LabelPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Label") {}
            text (text: "Renders an accessible label associated with controls.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add label") {}

            h2 (text: "Simple") {}
            preview-card (id: "label-basic") {
                label (text: "Email") {}
            }

            h2 (text: "With Input") {}
            preview-card (id: "label-input") {
                col (gap: "1.5", class: "w-64") {
                    label (text: "Email", for: "email") {}
                    input (id: "email", placeholder: "Enter your email") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "text") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Label text content (slot)") {}
                    }
                    tr {
                        td (text: "for") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "ID of the associated form control") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/label.at
git commit -m "feat(docs): improve label component page with input example and properties"
```

---

### Task 0.5: Update Tabs Page

**Files:**
- Modify: `examples/component-gallery/source/front/pages/tabs.at`

**Reference:** https://www.shadcn-vue.com/docs/components/tabs

**Current State:** Just a placeholder with no actual tabs demo.

**Step 1: Update tabs.at with complete content**

```auto
// pages/tabs.at - Tabs component documentation page

widget TabsPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Tabs") {}
            text (text: "A set of layered sections of content, known as tab panels, that display one panel of content at a time.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add tabs") {}

            h2 (text: "Simple") {}
            preview-card (id: "tabs-basic") {
                tabs (default: "account", class: "w-64") {
                    tabs-list {
                        tabs-trigger (text: "Account", value: "account") {}
                        tabs-trigger (text: "Password", value: "password") {}
                    }
                    tabs-content (value: "account") {
                        text (text: "Make changes to your account here.") {}
                    }
                    tabs-content (value: "password") {
                        text (text: "Change your password here.") {}
                    }
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "default") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Default active tab value") {}
                    }
                    tr {
                        td (text: "value") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Tab trigger/content value") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/tabs.at
git commit -m "feat(docs): improve tabs component page with working demo and properties"
```

---

### Task 0.6: Update Card Page

**Files:**
- Modify: `examples/component-gallery/source/front/pages/card.at`

**Reference:** https://www.shadcn-vue.com/docs/components/card

**Current State:** Has examples but missing Properties table and "Simple" section header.

**Step 1: Update card.at with Properties table**

```auto
// pages/card.at - Card component documentation page

widget CardPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Card") {}
            text (text: "Displays a card with header, content, and footer sections.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add card") {}

            h2 (text: "Simple") {}
            preview-card (id: "card-basic") {
                card (class: "w-[350px]") {
                    cardheader {
                        cardtitle (text: "Card Title") {}
                        carddescription (text: "Card Description") {}
                    }
                    cardcontent {
                        text (text: "Card content goes here.") {}
                    }
                    cardfooter {
                        button (text: "Action") {}
                    }
                }
            }

            h2 (text: "Examples") {}

            h3 (text: "Card with Form") {}
            preview-card (id: "card-form") {
                card (class: "w-[350px]") {
                    cardheader {
                        cardtitle (text: "Account") {}
                        carddescription (text: "Make changes to your account here.") {}
                    }
                    cardcontent {
                        col (gap: "4") {
                            col (gap: "1.5") {
                                label (text: "Name", for: "name") {}
                                input (id: "name", placeholder: "Pedro Duarte") {}
                            }
                        }
                    }
                    cardfooter {
                        button (text: "Save changes") {}
                    }
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "text") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Title/description text (slot)") {}
                    }
                    tr {
                        td (text: "class") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Additional CSS classes") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/card.at
git commit -m "feat(docs): improve card component page with properties table"
```

---

### Task 0.7: Update Text (Typography) Page

**Files:**
- Modify: `examples/component-gallery/source/front/pages/text.at`

**Reference:** https://www.shadcn-vue.com/docs/components/typography

**Current State:** Basic implementation without proper examples.

**Step 1: Update text.at with typography examples**

```auto
// pages/text.at - Text/Typography component documentation page

widget TextPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Typography") {}
            text (text: "Styled text components for headings, paragraphs, and other text elements.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add typography") {}

            h2 (text: "Examples") {}

            h3 (text: "Heading 1") {}
            preview-card (id: "text-h1") {
                h1 (text: "Heading Level 1", class: "scroll-m-20 text-4xl font-extrabold tracking-tight") {}
            }

            h3 (text: "Heading 2") {}
            preview-card (id: "text-h2") {
                h2 (text: "Heading Level 2", class: "scroll-m-20 text-3xl font-semibold tracking-tight") {}
            }

            h3 (text: "Paragraph") {}
            preview-card (id: "text-p") {
                text (text: "This is a paragraph of text. It can contain multiple sentences and will wrap naturally.", class: "leading-7") {}
            }

            h3 (text: "Lead") {}
            preview-card (id: "text-lead") {
                text (text: "A lead paragraph stands out from the rest.", class: "text-xl text-muted-foreground") {}
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "text") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Text content (slot)") {}
                    }
                    tr {
                        td (text: "class") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Tailwind classes for styling") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/text.at
git commit -m "feat(docs): improve typography component page with examples and properties"
```

---

## Batch 1: Simple Components (Create New Pages)

### Task 1: Alert Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/alert.at`

**Reference:** https://www.shadcn-vue.com/docs/components/alert

**Step 1: Create alert.at with basic structure**

```auto
// pages/alert.at - Alert component documentation page

widget AlertPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Alert") {}
            text (text: "Displays a callout for user attention.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add alert") {}

            h2 (text: "Simple") {}
            preview-card (id: "alert-basic") {
                alert {
                    text (text: "This is an alert message.") {}
                }
            }

            h2 (text: "Variants") {}
            preview-card (id: "alert-variants") {
                col (gap: "4") {
                    alert (variant: "default") {
                        text (text: "Default alert") {}
                    }
                    alert (variant: "destructive") {
                        text (text: "Error alert") {}
                    }
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "variant") {}
                        td (text: "string") {}
                        td (text: "default") {}
                        td (text: "default, destructive") {}
                        td (text: "Alert style variant") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/alert.at
git commit -m "feat(docs): add alert component page"
```

---

### Task 2: Avatar Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/avatar.at`

**Reference:** https://www.shadcn-vue.com/docs/components/avatar

**Step 1: Create avatar.at**

```auto
// pages/avatar.at - Avatar component documentation page

widget AvatarPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Avatar") {}
            text (text: "An image element with a fallback for representing the user.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add avatar") {}

            h2 (text: "Simple") {}
            preview-card (id: "avatar-basic") {
                row (gap: "4") {
                    avatar (src: "https://github.com/shadcn.png", alt: "@shadcn") {}
                    avatar (fallback: "CN") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "src") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Image source URL") {}
                    }
                    tr {
                        td (text: "alt") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Alt text for accessibility") {}
                    }
                    tr {
                        td (text: "fallback") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Fallback text when image fails") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/avatar.at
git commit -m "feat(docs): add avatar component page"
```

---

### Task 3: Breadcrumb Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/breadcrumb.at`

**Reference:** https://www.shadcn-vue.com/docs/components/breadcrumb

**Step 1: Create breadcrumb.at**

```auto
// pages/breadcrumb.at - Breadcrumb component documentation page

widget BreadcrumbPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Breadcrumb") {}
            text (text: "Shows the user's current location within a navigational hierarchy.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add breadcrumb") {}

            h2 (text: "Simple") {}
            preview-card (id: "breadcrumb-basic") {
                breadcrumb {
                    breadcrumb-item (text: "Home", href: "/") {}
                    breadcrumb-separator {}
                    breadcrumb-item (text: "Components", href: "/components") {}
                    breadcrumb-separator {}
                    breadcrumb-item (text: "Breadcrumb") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "text") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Item text content") {}
                    }
                    tr {
                        td (text: "href") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Link destination") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/breadcrumb.at
git commit -m "feat(docs): add breadcrumb component page"
```

---

### Task 4: Separator Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/separator.at`

**Reference:** https://www.shadcn-vue.com/docs/components/separator

**Step 1: Create separator.at**

```auto
// pages/separator.at - Separator component documentation page

widget SeparatorPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Separator") {}
            text (text: "Visually or semantically separates content.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add separator") {}

            h2 (text: "Horizontal") {}
            preview-card (id: "separator-horizontal") {
                col (gap: "4") {
                    text (text: "Content above") {}
                    separator (orientation: "horizontal") {}
                    text (text: "Content below") {}
                }
            }

            h2 (text: "Vertical") {}
            preview-card (id: "separator-vertical") {
                row (gap: "4", class: "h-8 items-center") {
                    text (text: "Left") {}
                    separator (orientation: "vertical") {}
                    text (text: "Right") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "orientation") {}
                        td (text: "string") {}
                        td (text: "horizontal") {}
                        td (text: "horizontal, vertical") {}
                        td (text: "Separator orientation") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/separator.at
git commit -m "feat(docs): add separator component page"
```

---

### Task 5: Skeleton Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/skeleton.at`

**Reference:** https://www.shadcn-vue.com/docs/components/skeleton

**Step 1: Create skeleton.at**

```auto
// pages/skeleton.at - Skeleton component documentation page

widget SkeletonPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Skeleton") {}
            text (text: "Use to show a placeholder while content is loading.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add skeleton") {}

            h2 (text: "Simple") {}
            preview-card (id: "skeleton-basic") {
                skeleton (class: "w-48 h-4") {}
            }

            h2 (text: "Card Skeleton") {}
            preview-card (id: "skeleton-card") {
                col (gap: "4", class: "w-64") {
                    skeleton (class: "h-12 w-12 rounded-full") {}
                    col (gap: "2") {
                        skeleton (class: "h-4 w-full") {}
                        skeleton (class: "h-4 w-3/4") {}
                    }
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "class") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Tailwind classes for sizing") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/skeleton.at
git commit -m "feat(docs): add skeleton component page"
```

---

### Task 6: Progress Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/progress.at`

**Reference:** https://www.shadcn-vue.com/docs/components/progress

**Step 1: Create progress.at**

```auto
// pages/progress.at - Progress component documentation page

widget ProgressPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Progress") {}
            text (text: "Displays an indicator showing the completion progress of a task.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add progress") {}

            h2 (text: "Simple") {}
            preview-card (id: "progress-basic") {
                progress (value: "33", class: "w-60") {}
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "value") {}
                        td (text: "number") {}
                        td (text: "0") {}
                        td (text: "0-100") {}
                        td (text: "Progress percentage") {}
                    }
                    tr {
                        td (text: "max") {}
                        td (text: "number") {}
                        td (text: "100") {}
                        td (text: "-") {}
                        td (text: "Maximum value") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/progress.at
git commit -m "feat(docs): add progress component page"
```

---

## Batch 2: Form Input Components

### Task 7: Textarea Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/textarea.at`

**Reference:** https://www.shadcn-vue.com/docs/components/textarea

**Step 1: Create textarea.at**

```auto
// pages/textarea.at - Textarea component documentation page

widget TextareaPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Textarea") {}
            text (text: "Displays a form textarea or a component that looks like a textarea.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add textarea") {}

            h2 (text: "Simple") {}
            preview-card (id: "textarea-basic") {
                textarea (placeholder: "Type your message here.", class: "w-64") {}
            }

            h2 (text: "Disabled") {}
            preview-card (id: "textarea-disabled") {
                textarea (placeholder: "Disabled textarea", disabled: "true", class: "w-64") {}
            }

            h2 (text: "With Label") {}
            preview-card (id: "textarea-label") {
                col (gap: "1.5", class: "w-64") {
                    label (text: "Bio", for: "bio") {}
                    textarea (id: "bio", placeholder: "Tell us about yourself") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "placeholder") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Placeholder text") {}
                    }
                    tr {
                        td (text: "disabled") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Disables the textarea") {}
                    }
                    tr {
                        td (text: "rows") {}
                        td (text: "number") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Number of visible text lines") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/textarea.at
git commit -m "feat(docs): add textarea component page"
```

---

### Task 8: Select Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/select.at`

**Reference:** https://www.shadcn-vue.com/docs/components/select

**Step 1: Create select.at**

```auto
// pages/select.at - Select component documentation page

widget SelectPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Select") {}
            text (text: "Displays a list of options for the user to pick from, triggered by a button.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add select") {}

            h2 (text: "Simple") {}
            preview-card (id: "select-basic") {
                select (placeholder: "Select a fruit") {
                    select-item (text: "Apple", value: "apple") {}
                    select-item (text: "Banana", value: "banana") {}
                    select-item (text: "Orange", value: "orange") {}
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "placeholder") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Placeholder text") {}
                    }
                    tr {
                        td (text: "disabled") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Disables the select") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/select.at
git commit -m "feat(docs): add select component page"
```

---

### Task 9: Switch Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/switch.at`

**Reference:** https://www.shadcn-vue.com/docs/components/switch

**Step 1: Create switch.at**

```auto
// pages/switch.at - Switch component documentation page

widget SwitchPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Switch") {}
            text (text: "A control that allows the user to toggle between checked and not checked.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add switch") {}

            h2 (text: "Simple") {}
            preview-card (id: "switch-basic") {
                switch {}
            }

            h2 (text: "With Label") {}
            preview-card (id: "switch-label") {
                row (gap: "2", class: "items-center") {
                    switch (id: "airplane-mode") {}
                    label (text: "Airplane Mode", for: "airplane-mode") {}
                }
            }

            h2 (text: "Disabled") {}
            preview-card (id: "switch-disabled") {
                switch (disabled: "true") {}
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "checked") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Switch state") {}
                    }
                    tr {
                        td (text: "disabled") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Disables the switch") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/switch.at
git commit -m "feat(docs): add switch component page"
```

---

### Task 10: Slider Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/slider.at`

**Reference:** https://www.shadcn-vue.com/docs/components/slider

**Step 1: Create slider.at**

```auto
// pages/slider.at - Slider component documentation page

widget SliderPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "Slider") {}
            text (text: "An input where the user selects a value from within a given range.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add slider") {}

            h2 (text: "Simple") {}
            preview-card (id: "slider-basic") {
                slider (min: "0", max: "100", value: "50", class: "w-60") {}
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "min") {}
                        td (text: "number") {}
                        td (text: "0") {}
                        td (text: "-") {}
                        td (text: "Minimum value") {}
                    }
                    tr {
                        td (text: "max") {}
                        td (text: "number") {}
                        td (text: "100") {}
                        td (text: "-") {}
                        td (text: "Maximum value") {}
                    }
                    tr {
                        td (text: "value") {}
                        td (text: "number") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Current value") {}
                    }
                    tr {
                        td (text: "step") {}
                        td (text: "number") {}
                        td (text: "1") {}
                        td (text: "-") {}
                        td (text: "Step increment") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/slider.at
git commit -m "feat(docs): add slider component page"
```

---

### Task 11: RadioGroup Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/radiogroup.at`

**Reference:** https://www.shadcn-vue.com/docs/components/radio-group

**Step 1: Create radiogroup.at**

```auto
// pages/radiogroup.at - RadioGroup component documentation page

widget RadioGroupPage {
    model { activeTab str = "preview" }

    view {
        col {
            h1 (text: "RadioGroup") {}
            text (text: "A set of checkable buttons, known as radio buttons, where no more than one of the buttons can be checked at a time.") {}

            h2 (text: "Installation") {}
            codeblock (lang: "bash", code: "npx shadcn-vue@latest add radio-group") {}

            h2 (text: "Simple") {}
            preview-card (id: "radiogroup-basic") {
                radiogroup (default: "option-one") {
                    row (gap: "2", class: "items-center") {
                        radioitem (value: "option-one", id: "option-one") {}
                        label (text: "Option One", for: "option-one") {}
                    }
                    row (gap: "2", class: "items-center") {
                        radioitem (value: "option-two", id: "option-two") {}
                        label (text: "Option Two", for: "option-two") {}
                    }
                }
            }

            h2 (text: "Properties") {}
            table {
                thead {
                    tr {
                        th (text: "Property") {}
                        th (text: "Type") {}
                        th (text: "Default") {}
                        th (text: "Values") {}
                        th (text: "Description") {}
                    }
                }
                tbody {
                    tr {
                        td (text: "value") {}
                        td (text: "string") {}
                        td (text: "-") {}
                        td (text: "-") {}
                        td (text: "Selected radio value") {}
                    }
                    tr {
                        td (text: "disabled") {}
                        td (text: "boolean") {}
                        td (text: "false") {}
                        td (text: "true, false") {}
                        td (text: "Disables the group") {}
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/component-gallery/source/front/pages/radiogroup.at
git commit -m "feat(docs): add radiogroup component page"
```

---

## Batch 3: Navigation Components

### Task 12: Pagination Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/pagination.at`

**Reference:** https://www.shadcn-vue.com/docs/components/pagination

**Content:** Pagination with page numbers, prev/next buttons, ellipsis for large ranges.

---

### Task 13: Menubar Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/menubar.at`

**Reference:** https://www.shadcn-vue.com/docs/components/menubar

**Content:** Horizontal menu bar with dropdown menus, keyboard navigation.

---

### Task 14: NavigationMenu Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/navigationmenu.at`

**Reference:** https://www.shadcn-vue.com/docs/components/navigation-menu

**Content:** Collection of navigation links with dropdown support.

---

### Task 15: DropdownMenu Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/dropdownmenu.at`

**Reference:** https://www.shadcn-vue.com/docs/components/dropdown-menu

**Content:** Menu triggered by a button, with items, separators, and submenus.

---

### Task 16: ContextMenu Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/contextmenu.at`

**Reference:** https://www.shadcn-vue.com/docs/components/context-menu

**Content:** Right-click context menu with items and submenus.

---

## Batch 4: Overlay Components

### Task 17: Dialog Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/dialog.at`

**Reference:** https://www.shadcn-vue.com/docs/components/dialog

**Content:** Modal dialog with header, content, footer, close button.

---

### Task 18: AlertDialog Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/alertdialog.at`

**Reference:** https://www.shadcn-vue.com/docs/components/alert-dialog

**Content:** Modal alert for confirmations, with cancel and action buttons.

---

### Task 19: Drawer Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/drawer.at`

**Reference:** https://www.shadcn-vue.com/docs/components/drawer

**Content:** Side drawer panel that slides in from left/right/top/bottom.

---

### Task 20: Sheet Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/sheet.at`

**Reference:** https://www.shadcn-vue.com/docs/components/sheet

**Content:** Side panel that slides in, similar to drawer but more general purpose.

---

### Task 21: Popover Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/popover.at`

**Reference:** https://www.shadcn-vue.com/docs/components/popover

**Content:** Floating content that appears near a trigger element.

---

### Task 22: HoverCard Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/hovercard.at`

**Reference:** https://www.shadcn-vue.com/docs/components/hover-card

**Content:** Card that appears on hover, like user profile preview.

---

### Task 23: Tooltip Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/tooltip.at`

**Reference:** https://www.shadcn-vue.com/docs/components/tooltip

**Content:** Small text popup that appears on hover over an element.

---

## Batch 5: Complex Components

### Task 24: Table Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/table.at`

**Reference:** https://www.shadcn-vue.com/docs/components/table

**Content:** Data table with headers, rows, cells, sortable columns.

---

### Task 25: DataTable Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/datatable.at`

**Reference:** https://www.shadcn-vue.com/docs/components/data-table

**Content:** Advanced data table with filtering, sorting, pagination.

---

### Task 26: Calendar Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/calendar.at`

**Reference:** https://www.shadcn-vue.com/docs/components/calendar

**Content:** Date calendar with month navigation, date selection.

---

### Task 27: DatePicker Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/datepicker.at`

**Reference:** https://www.shadcn-vue.com/docs/components/date-picker

**Content:** Date input with calendar popover for selection.

---

### Task 28: Carousel Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/carousel.at`

**Reference:** https://www.shadcn-vue.com/docs/components/carousel

**Content:** Image/content carousel with navigation, autoplay, indicators.

---

### Task 29: Collapsible Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/collapsible.at`

**Reference:** https://www.shadcn-vue.com/docs/components/collapsible

**Content:** Expandable/collapsible content section.

---

### Task 30: Combobox Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/combobox.at`

**Reference:** https://www.shadcn-vue.com/docs/components/combobox

**Content:** Combination of input and dropdown for autocomplete selection.

---

### Task 31: Command Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/command.at`

**Reference:** https://www.shadcn-vue.com/docs/components/command

**Content:** Command palette for quick actions, searchable list.

---

### Task 32: Form Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/form.at`

**Reference:** https://www.shadcn-vue.com/docs/components/form

**Content:** Form component with validation, field components, error handling.

---

## Batch 6: Feedback Components

### Task 33: Toast Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/toast.at`

**Reference:** https://www.shadcn-vue.com/docs/components/toast

**Content:** Brief notification message that appears temporarily.

---

### Task 34: Sonner Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/sonner.at`

**Reference:** https://www.shadcn-vue.com/docs/components/sonner

**Content:** Alternative toast notification library integration.

---

### Task 35: Toggle Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/toggle.at`

**Reference:** https://www.shadcn-vue.com/docs/components/toggle

**Content:** Two-state button that can be toggled on/off.

---

### Task 36: ToggleGroup Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/togglegroup.at`

**Reference:** https://www.shadcn-vue.com/docs/components/toggle-group

**Content:** Group of toggle buttons with single or multiple selection.

---

### Task 37: ScrollArea Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/scrollarea.at`

**Reference:** https://www.shadcn-vue.com/docs/components/scroll-area

**Content:** Custom scrollable container with styled scrollbars.

---

### Task 38: Sidebar Widget Page

**Files:**
- Create: `examples/component-gallery/source/front/pages/sidebar.at`

**Reference:** https://www.shadcn-vue.com/docs/components/sidebar

**Content:** Composable sidebar component with sections, collapsible groups.

---

## Verification

After completing all tasks, verify:

```bash
# List all page files
ls examples/component-gallery/source/front/pages/*.at | wc -l
# Expected: 48 (10 existing, 7 updated + 38 new)

# Regenerate Vue output
cargo run --release -- ui examples/component-gallery/source/front

# Verify all routes work
# Open http://localhost:5173/ in browser and test navigation
```

---

## Task Summary

| Batch | Description | Tasks |
|-------|-------------|-------|
| Batch 0 | Update existing crude pages | 0.1-0.7 (7 tasks) |
| Batch 1 | Simple components (create new) | 1-6 (6 tasks) |
| Batch 2 | Form input components | 7-11 (5 tasks) |
| Batch 3 | Navigation components | 12-16 (5 tasks) |
| Batch 4 | Overlay components | 17-23 (7 tasks) |
| Batch 5 | Complex components | 24-32 (9 tasks) |
| Batch 6 | Feedback components | 33-38 (6 tasks) |
| **Total** | | **45 tasks** |

---

## Notes

- Each task requires checking shadcn-vue documentation for accurate props and examples
- Properties tables should include: Property, Type, Default, Values, Description columns
- Use `preview-card` for all interactive examples
- Use `codeblock` for installation commands
- Maintain alphabetical ordering in sidebar and index
- Batch 0 tasks update existing files, other batches create new files
