---
title: 核心组件参考(Core Components)
---

> 本页由 schema/aura.at 生成(Plan 435 P5)—— **勿手改**;再生成:
> `auto docs gen`(推荐;测试内等价 `DOCS_GEN_UPDATE=1 cargo test -p auto-lang --test docs_gen`)
> tier 语义:`builtin_widget`=桌面有实现;`native_html`=Web 原生直通。
> shadcn 家族组件的活文档/Demo 见 widgets-gallery(本页仅收核心层)。

## 内置组件(builtin_widget)

### `article`

`builtin_widget` · `article` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`Article`

_props 待声明_

---

### `aside`

`builtin_widget` · `aside` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`Aside`

_props 待声明_

---

### `autodown_editor`

`builtin_widget` · `autodown_editor` · web: `component` · iced: `unknown` · category: `form`

AutoDown block editor (plan 019 Phase 3 shell; @autodown/engine AutoDownEditor on vue, cosmic-text block buffers on VM)

别名:`autodowneditor`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `key` | `string` | — | Stable state-storage identity (VM editor shell registry key) |
| `content` | `union: string|state_ref` | — | Bound markdown document body |
| `final` | `union: bool|state_ref` | true | Streaming marker (editor treats document as final) |
| `can_edit` | `bool` | true | Whether the editor is interactive (vue arm: canEdit) |
| `show_actions` | `bool` | true | Show editor action bar (vue arm: showActions) |
| `oninput` | `msg_ref` | — | Message on document edit (payload via autodown_editor_text(key) on VM) |
| `onchange` | `msg_ref` | — | Alias of oninput |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `avatar`

`builtin_widget` · `avatar` · web: `component` · iced: `partial` · category: `content`

[demo →](/examples/widgets-gallery/avatar)

P1 extracted from production tables; props TBD

别名:`Avatar`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `src` | `string` | — | Image source URL |
| `alt` | `string` | — | Alt text for accessibility |
| `fallback` | `string` | — | Fallback text when image fails |

子件:`avatarfallback` `avatarimage`

---

### `b`

`builtin_widget` · `b` · web: `none` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `badge`

`builtin_widget` · `badge` · web: `component` · iced: `partial` · category: `feedback`

[demo →](/examples/widgets-gallery/badge)

Badge for status or labels

别名:`Badge`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `text` | `string` | — | Badge text |
| `variant` | `one_of: default|secondary|destructive|outline` | default | Badge variant |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `button`

`builtin_widget` · `button` · web: `component` · iced: `full` · category: `content`

[demo →](/examples/widgets-gallery/button)

A clickable button element

别名:`Button` `btn`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `text` | `string` | — | Button label text |
| `onclick` | `msg_ref` | — | Message to send when clicked |
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `disabled` | `union: bool|state_ref` | false | Whether button is disabled |
| `variant` | `one_of: default|secondary|destructive|outline|ghost|link` | default | Visual style variant |
| `size` | `one_of: sm|default|lg|icon` | default | Button size |
| `icon` | `string` | — | Icon name shown alongside the label |

---

### `center`

`builtin_widget` · `center` · web: `component` · iced: `full` · category: `content`

[demo →](/examples/widgets-gallery/center)

P1 extracted from production tables; props TBD

别名:`Center`

_props 待声明_

---

### `checkbox`

`builtin_widget` · `checkbox` · web: `component` · iced: `full` · category: `content`

[demo →](/examples/widgets-gallery/checkbox)

Checkbox control

别名:`Checkbox` `check`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `checked` | `union: bool|state_ref` | false | Checked state |
| `onchange` | `msg_ref` | — | Message on toggle |
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `disabled` | `union: bool|state_ref` | false | Whether checkbox is disabled |

---

### `code_editor`

`builtin_widget` · `code_editor` · web: `component` · iced: `full` · category: `content`

[demo →](/examples/widgets-gallery/code-editor)

P1 extracted from production tables; props TBD

别名:`codeEditor` `codeeditor`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `key` | `string` | — | Stable state key (required for value persistence) |
| `content` | `string` | — | External value; only rewritten on diff |
| `lang` | `one_of: rust|python|auto|none|...` | none | Syntax highlighting language |
| `line_numbers` | `one_of: true|false` | true | Show the line-number gutter |
| `wrap` | `one_of: true|false` | false | Soft wrap long lines |
| `vi` | `one_of: true|false` | false | Vi mode (iced only; the vue CodeMirror shell ignores :vi) |
| `search` | `one_of: regex` | — | Live regex highlight + jump (search-as-you-type) |
| `tab_width` | `int` | 4 | Tab width in columns |

---

### `col`

`builtin_widget` · `col` · web: `component` · iced: `full` · category: `layout`

[demo →](/examples/widgets-gallery/col)

Vertical layout container

别名:`Col` `Column` `column`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `gap` | `int` | 0 | Spacing between children |
| `padding` | `union: int|string` | 0 | Inner padding |
| `align` | `one_of: start|center|end|stretch` | start | Cross-axis alignment |

---

### `container`

`builtin_widget` · `container` · web: `native` · iced: `full` · category: `layout`

Generic container with optional constraints

别名:`Container` `div`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `max_width` | `int` | — | Maximum width in pixels |
| `padding` | `union: int|string` | — | Inner padding |

---

### `divider`

`builtin_widget` · `divider` · web: `native` · iced: `partial` · category: `utility`

Horizontal or vertical divider line

别名:`Divider` `hr`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `direction` | `one_of: horizontal|vertical` | horizontal | Divider direction |

---

### `em`

`builtin_widget` · `em` · web: `none` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `footer`

`builtin_widget` · `footer` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`Footer`

_props 待声明_

---

### `grid`

`builtin_widget` · `grid` · web: `component` · iced: `partial` · category: `layout`

[demo →](/examples/widgets-gallery/grid)

Grid layout container

别名:`Grid`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `columns` | `int` | 1 | Number of columns |
| `gap` | `int` | 0 | Cell spacing |

---

### `h1`

`builtin_widget` · `h1` · web: `native` · iced: `full` · category: `content`

P1 extracted from production tables; props TBD

别名:`H1`

_props 待声明_

---

### `h2`

`builtin_widget` · `h2` · web: `native` · iced: `full` · category: `content`

P1 extracted from production tables; props TBD

别名:`H2`

_props 待声明_

---

### `h3`

`builtin_widget` · `h3` · web: `native` · iced: `full` · category: `content`

P1 extracted from production tables; props TBD

别名:`H3`

_props 待声明_

---

### `h4`

`builtin_widget` · `h4` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`H4`

_props 待声明_

---

### `h5`

`builtin_widget` · `h5` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`H5`

_props 待声明_

---

### `h6`

`builtin_widget` · `h6` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`H6`

_props 待声明_

---

### `header`

`builtin_widget` · `header` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`Header`

_props 待声明_

---

### `i`

`builtin_widget` · `i` · web: `none` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `icon`

`builtin_widget` · `icon` · web: `native` · iced: `unknown` · category: `media`

Icon display

别名:`Icon`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `name` | `string` | — | Icon name |
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `size` | `int` | 24 | Icon size in pixels |

---

### `image`

`builtin_widget` · `image` · web: `component` · iced: `partial` · category: `media`

Image display

别名:`Image`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `src` | `string` | — | Image URL |
| `alt` | `string` |  | Alt text |
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `fit` | `one_of: cover|contain|fill|none` | cover | Object fit mode |

---

### `img`

`builtin_widget` · `img` · web: `native` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

别名:`Img`

_props 待声明_

---

### `input`

`builtin_widget` · `input` · web: `component` · iced: `partial` · category: `content`

[demo →](/examples/widgets-gallery/input)

Text input field

别名:`Input`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `state_ref` | — | Bound value (two-way binding) |
| `placeholder` | `string` | — | Placeholder text |
| `type` | `one_of: text|password|email|number` | text | Input type |
| `onchange` | `msg_ref` | — | Message on value change |
| `onenter` | `msg_ref` | — | Message on Enter key |
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `disabled` | `union: bool|state_ref` | false | Whether input is disabled |

---

### `label`

`builtin_widget` · `label` · web: `component` · iced: `full` · category: `form`

[demo →](/examples/widgets-gallery/label)

Form label

别名:`Label`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `for` | `string` | — | Associated form control ID |
| `text` | `string` | — | Label text |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `link`

`builtin_widget` · `link` · web: `native` · iced: `unknown` · category: `content`

Hyperlink

别名:`Link`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `href` | `string` | — | Link URL |
| `text` | `string` | — | Link text |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `list`

`builtin_widget` · `list` · web: `component` · iced: `fallback` · category: `list`

Generic list container

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `list_item`

`builtin_widget` · `list_item` · web: `component` · iced: `fallback` · category: `list`

List item

别名:`list-item` `listitem`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `onclick` | `msg_ref` | — | Message when clicked |

---

### `main`

`builtin_widget` · `main` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`Main`

_props 待声明_

---

### `markdown`

`builtin_widget` · `markdown` · web: `component` · iced: `unknown` · category: `content`

AutoDown document renderer (read-only; @autodown/engine MarkdownRender on vue, autodown-core parse_blocks on VM)

别名:`autodown` `markdown_editor`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `content` | `union: string|state_ref` | — | Markdown source (literal or state-bound) |
| `final` | `union: bool|state_ref` | true | Streaming marker: false = still receiving chunks (dangling-marker stripping) |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `menubar`

`builtin_widget` · `menubar` · web: `component` · iced: `unknown` · category: `navigation`

[demo →](/examples/widgets-gallery/menubar)

Menubar container

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

子件:`menubar_content` `menubar_item` `menubar_menu` `menubar_separator` `menubar_trigger`

---

### `nav`

`builtin_widget` · `nav` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`Nav`

_props 待声明_

---

### `nav-link`

`builtin_widget` · `nav-link` · web: `component` · iced: `partial` · category: `content`

[demo →](/examples/widgets-gallery/navlink)

P1 extracted from production tables; props TBD

别名:`nav_link` `navlink`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `text` | `one_of: href` | Link label text | string |
| `icon` | `one_of: badge` | Lucide icon name | string |

---

### `p`

`builtin_widget` · `p` · web: `native` · iced: `full` · category: `typography`

Paragraph text

别名:`P`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `text` | `string` | — | Paragraph text |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `popover`

`builtin_widget` · `popover` · web: `component` · iced: `fallback` · category: `navigation`

[demo →](/examples/widgets-gallery/popover)

Anchored popover (overlay)

别名:`Popover`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `expr` | — | Open state (state binding, e.g. .ctx_open) |
| `x` | `expr` | — | Coordinate anchor x (viewport px, contextmenu form) |
| `y` | `expr` | — | Coordinate anchor y (viewport px, contextmenu form) |
| `placement` | `one_of: bottom|bottom-start|bottom-end|top|top-start|top-end|left|right` | bottom-start | Panel placement relative to anchor |
| `ondismiss` | `msg_ref` | — | Fired on outside click / anchor click / Esc / focus loss |
| `class` | `union: string|class_binding` | — | Panel chrome classes (bg/border/shadow land on the panel) |

子件:`popover_content` `popover_trigger`

---

### `progress`

`builtin_widget` · `progress` · web: `component` · iced: `full` · category: `content`

[demo →](/examples/widgets-gallery/progress)

P1 extracted from production tables; props TBD

别名:`Progress`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `float` | 0 | Progress percentage |
| `max` | `float` | 100 | Maximum value |

---

### `row`

`builtin_widget` · `row` · web: `component` · iced: `full` · category: `layout`

[demo →](/examples/widgets-gallery/row)

Horizontal layout container

别名:`Row`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `gap` | `int` | 0 | Spacing between children |
| `padding` | `union: int|string` | 0 | Inner padding |
| `align` | `one_of: start|center|end|stretch` | center | Cross-axis alignment |

---

### `scroll`

`builtin_widget` · `scroll` · web: `component` · iced: `partial` · category: `layout`

[demo →](/examples/widgets-gallery/scroll)

Scrollable container

别名:`Scroll` `scrollable`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `direction` | `one_of: vertical|horizontal|both` | vertical | Scroll direction |

子件:`scrollarea` `scrollareascrollbar` `scrollareathumb` `scrollareaviewport`

---

### `section`

`builtin_widget` · `section` · web: `native` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

别名:`Section`

_props 待声明_

---

### `separator`

`builtin_widget` · `separator` · web: `component` · iced: `unknown` · category: `utility`

[demo →](/examples/widgets-gallery/separator)

Visual divider

别名:`Separator` `sep`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `orientation` | `one_of: horizontal|vertical` | horizontal | Separator orientation |
| `label` | `string` | — | Optional label for separator |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `small`

`builtin_widget` · `small` · web: `none` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `spacer`

`builtin_widget` · `spacer` · web: `component` · iced: `full` · category: `utility`

Flexible or fixed space

别名:`Spacer`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `size` | `int` | — | Spacer size in pixels (or flex if omitted) |

---

### `span`

`builtin_widget` · `span` · web: `native` · iced: `full` · category: `typography`

Inline text span

别名:`Span`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `text` | `string` | — | Span text |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `square`

`builtin_widget` · `square` · web: `native` · iced: `full` · category: `content`

P1 extracted from production tables; props TBD

别名:`Square`

_props 待声明_

---

### `strong`

`builtin_widget` · `strong` · web: `none` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `svg`

`builtin_widget` · `svg` · web: `native` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `taskbar`

`builtin_widget` · `taskbar` · web: `none` · iced: `full` · category: `navigation`

Desktop shell taskbar (bottom bar)

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | Bar chrome classes (h-/w-/bg-/border- land on the bar) |

---

### `text`

`builtin_widget` · `text` · web: `component` · iced: `full` · category: `typography`

Text content (literal or interpolated)

别名:`Text`

_props 待声明_

---

### `textarea`

`builtin_widget` · `textarea` · web: `component` · iced: `partial` · category: `form`

[demo →](/examples/widgets-gallery/textarea)

Multi-line text input

别名:`Textarea`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `placeholder` | `string` | — | Placeholder text |
| `value` | `union: string|state_ref` | — | Textarea value |
| `disabled` | `bool` | false | Disabled state |
| `rows` | `int` | — | Number of rows |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `toast-provider`

`builtin_widget` · `toast-provider` · web: `component` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

别名:`toast_provider` `toastprovider`

_props 待声明_

---

### `toaster`

`builtin_widget` · `toaster` · web: `none` · iced: `partial` · category: `content`

Toast notification container (alias)

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `position` | `one_of: top-left|top-center|top-right|bottom-left|bottom-center|bottom-right` | bottom-right | Toast position |

---

### `toolbar`

`builtin_widget` · `toolbar` · web: `none` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

## 原生直通(native_html)

### `+`

`native_html` · `+` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `-`

`native_html` · `-` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `a`

`native_html` · `a` · web: `none` · iced: `unknown` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `audio`

`native_html` · `audio` · web: `none` · iced: `fallback` · category: `content`

P1 extracted from production tables; props TBD

别名:`Audio`

_props 待声明_

---

### `blockquote`

`native_html` · `blockquote` · web: `native` · iced: `none` · category: `content`

Block quotation

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `canvas`

`native_html` · `canvas` · web: `none` · iced: `fallback` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `circle`

`native_html` · `circle` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `clipPath`

`native_html` · `clipPath` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `code`

`native_html` · `code` · web: `native` · iced: `partial` · category: `typography`

Inline code

别名:`Code`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `codeblock`

`native_html` · `codeblock` · web: `native` · iced: `partial` · category: `content`

Code block with syntax highlighting

别名:`CodeBlock` `Codeblock` `code-block` `code_block`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `lang` | `string` | text | Programming language for syntax highlighting |
| `code` | `string` | — | Code content |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `codepane`

`native_html` · `codepane` · web: `native` · iced: `partial` · category: `content`

Tabbed code block showing Auto and Vue code side by side

别名:`CodePane` `code-pane`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `auto` | `string` | — | Auto (AURA) source code |
| `vue` | `string` | — | Generated Vue.js code |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `dd`

`native_html` · `dd` · web: `native` · iced: `none` · category: `list`

Description detail

别名:`Dd`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `defs`

`native_html` · `defs` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `dl`

`native_html` · `dl` · web: `native` · iced: `none` · category: `list`

Description list

别名:`Dl`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `dt`

`native_html` · `dt` · web: `native` · iced: `none` · category: `list`

Description term

别名:`Dt`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `ellipse`

`native_html` · `ellipse` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `figcaption`

`native_html` · `figcaption` · web: `native` · iced: `none` · category: `content`

Figure caption

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `figure`

`native_html` · `figure` · web: `native` · iced: `none` · category: `content`

Figure container

别名:`Figure`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `g`

`native_html` · `g` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `li`

`native_html` · `li` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

别名:`Li`

_props 待声明_

---

### `line`

`native_html` · `line` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `linearGradient`

`native_html` · `linearGradient` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `mask`

`native_html` · `mask` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `native_button`

`native_html` · `native_button` · web: `native` · iced: `none` · category: `content`

Native HTML button escape (bypasses button-to-Button mapping)

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `text` | `string` | — | Button label text |
| `onclick` | `msg_ref` | — | Message to send when clicked |
| `class` | `union: string|class_binding` | — | CSS class(es) |
| `disabled` | `bool` | false | Whether button is disabled |

---

### `ol`

`native_html` · `ol` · web: `native` · iced: `none` · category: `list`

Ordered list

别名:`Ol`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `optgroup`

`native_html` · `optgroup` · web: `native` · iced: `none` · category: `form`

Option group

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `label` | `string` | — | Group label |
| `disabled` | `bool` | false | Disabled state |

---

### `option`

`native_html` · `option` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

别名:`Option`

_props 待声明_

---

### `path`

`native_html` · `path` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `polygon`

`native_html` · `polygon` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `polyline`

`native_html` · `polyline` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `pre`

`native_html` · `pre` · web: `native` · iced: `none` · category: `typography`

Preformatted text block

别名:`Pre`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `previewcard`

`native_html` · `previewcard` · web: `native` · iced: `partial` · category: `content`

Preview card with collapsible code section - like shadcn-vue docs

别名:`PreviewCard` `preview-card` `preview_card`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `id` | `string` | — | Unique identifier for the preview card (used for state variables) |
| `title` | `string` | Preview | Section title |
| `auto` | `string` | — | Auto (AURA) source code |
| `vue` | `string` | — | Generated Vue.js code |
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `radialGradient`

`native_html` · `radialGradient` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `radio`

`native_html` · `radio` · web: `native` · iced: `fallback` · category: `content`

P1 extracted from production tables; props TBD

别名:`Radio`

_props 待声明_

---

### `rect`

`native_html` · `rect` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `spinner`

`native_html` · `spinner` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

别名:`Spinner`

_props 待声明_

---

### `stop`

`native_html` · `stop` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `summary`

`native_html` · `summary` · web: `none` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

别名:`Summary`

_props 待声明_

---

### `tab`

`native_html` · `tab` · web: `native` · iced: `fallback` · category: `content`

P1 extracted from production tables; props TBD

别名:`Tab`

_props 待声明_

---

### `tbody`

`native_html` · `tbody` · web: `native` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

别名:`Tbody`

_props 待声明_

---

### `td`

`native_html` · `td` · web: `native` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

别名:`Td`

_props 待声明_

---

### `tfoot`

`native_html` · `tfoot` · web: `none` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `th`

`native_html` · `th` · web: `native` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

别名:`Th`

_props 待声明_

---

### `thead`

`native_html` · `thead` · web: `native` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

别名:`Thead`

_props 待声明_

---

### `tr`

`native_html` · `tr` · web: `native` · iced: `partial` · category: `content`

P1 extracted from production tables; props TBD

别名:`Tr`

_props 待声明_

---

### `tree`

`native_html` · `tree` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

别名:`Tree`

_props 待声明_

---

### `tree-item`

`native_html` · `tree-item` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

别名:`TreeItem` `tree_item`

_props 待声明_

---

### `ul`

`native_html` · `ul` · web: `native` · iced: `none` · category: `list`

Unordered list

别名:`Ul`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `class` | `union: string|class_binding` | — | CSS class(es) |

---

### `use`

`native_html` · `use` · web: `native` · iced: `none` · category: `content`

P1 extracted from production tables; props TBD

_props 待声明_

---

### `video`

`native_html` · `video` · web: `none` · iced: `fallback` · category: `content`

P1 extracted from production tables; props TBD

别名:`Video`

_props 待声明_

---

