# 026-keyboard-mouse-events

Demonstrates the generic DOM event capabilities of the AURA widget system's
Vue generator:

- **Keyboard navigation (SlashMenu style)** — `onkeydown: .Nav($event)` passes
  the DOM event object so the handler can read `e.key`; key modifiers
  (`onkeydown.enter`, `onkeydown.escape`) map to Vue's `@keydown.enter` /
  `@keydown.esc`.
- **Drag tracking (CustomScrollbar style)** — `onmousedown: .StartDrag($event.clientY)`
  on the thumb, plus `onmousemove.window` / `onmouseup.window` on the root:
  window-level listeners (emitted as `window.addEventListener` in `onMounted`
  with matching `removeEventListener` in `onUnmounted`) keep tracking the drag
  outside the element.
- **Scroll lock (CodeBlockMenu style)** — `onwheel.document.capture.prevent`
  registers a document-level wheel listener in the capture phase; the
  generated wrapper calls `e.preventDefault()` and the listener is registered
  with `{ capture: true, passive: false }` (Chrome requires `passive: false`
  for `preventDefault` on document-level wheel listeners).

## Event syntax summary

```auto
view {
    col {
        onkeydown: .Nav($event),                    // generic event + event object
        onkeydown.enter: .Pick,                     // key modifier
        oncontextmenu.prevent: .Ctx,                // event modifier (preventDefault)
        onclick.stop: .Tap,                         // event modifier (stopPropagation)
        onmousemove.window: .DragMove($event),      // window-level listener
        onwheel.document.capture.prevent: .Lock($event.deltaY), // document-level, capture+prevent
    }
}
```

## Build

```sh
auto build   # generates gen/front/vue and runs vue-tsc + vite build
```

See `src/front/app.at` for the full source.
