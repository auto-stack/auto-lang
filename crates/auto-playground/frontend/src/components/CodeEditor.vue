<template>
  <div ref="editorContainer" class="editor-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from 'vue';
import { EditorState, type Extension, Compartment, StateEffect, StateField, RangeSetBuilder } from '@codemirror/state';
import { EditorView, keymap, lineNumbers, highlightActiveLine, Decoration, type DecorationSet, GutterMarker, gutter } from '@codemirror/view';
import { defaultKeymap, indentWithTab, history, historyKeymap } from '@codemirror/commands';
import { oneDark } from '@codemirror/theme-one-dark';
import { autoLanguage } from '../lang/auto';

const props = defineProps<{
  modelValue: string;
  onRun?: () => void;
  isDebugging?: boolean;
  breakpoints?: number[];
  currentDebugLine?: number | null;
  highlightedSourceLine?: number | null;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'line-click': [line: number];
  'breakpointsChange': [lines: number[]];
  'hover-line': [line: number];
  'hover-line-leave': [];
}>();

const editorContainer = ref<HTMLDivElement>();
let editorView: EditorView | null = null;
const debugCompartment = new Compartment();

// ============================================================================
// Breakpoint Gutter
// ============================================================================

const breakpointEffect = StateEffect.define<number>();

const breakpointState = StateField.define<Set<number>>({
  create() { return new Set(); },
  update(set, tr) {
    for (const e of tr.effects) {
      if (e.is(breakpointEffect)) {
        const line = e.value;
        const newSet = new Set(set);
        if (newSet.has(line)) newSet.delete(line);
        else newSet.add(line);
        return newSet;
      }
    }
    return set;
  },
});

class BreakpointMarker extends GutterMarker {
  eq(other: GutterMarker): boolean { return other instanceof BreakpointMarker; }
  override toDOM() {
    const el = document.createElement('div');
    el.style.width = '10px';
    el.style.height = '10px';
    el.style.borderRadius = '50%';
    el.style.background = '#e51400';
    el.className = 'cm-breakpoint-marker';
    return el;
  }
}

class EmptyCircleMarker extends GutterMarker {
  eq(other: GutterMarker): boolean { return other instanceof EmptyCircleMarker; }
  override toDOM() {
    const el = document.createElement('div');
    el.style.width = '10px';
    el.style.height = '10px';
    el.style.borderRadius = '50%';
    el.style.border = '1.5px solid #e51400';
    el.style.background = 'transparent';
    el.style.opacity = '0';
    el.style.transition = 'opacity 0.15s ease';
    el.className = 'cm-empty-circle-marker';
    return el;
  }
}

class SpacerMarker extends GutterMarker {
  eq(other: GutterMarker): boolean { return other instanceof SpacerMarker; }
  override toDOM() {
    const el = document.createElement('div');
    el.style.width = '22px';
    el.className = 'cm-breakpoint-spacer';
    return el;
  }
}

const breakpointGutter = [
  breakpointState,
  gutter({
    class: 'cm-breakpoint-gutter',
    markers(view) {
      const builder = new RangeSetBuilder<GutterMarker>();
      const bps = view.state.field(breakpointState);
      for (let i = 1; i <= view.state.doc.lines; i++) {
        const line = view.state.doc.line(i);
        const marker = bps.has(i) ? new BreakpointMarker() : new EmptyCircleMarker();
        builder.add(line.from, line.from, marker);
      }
      return builder.finish();
    },
    initialSpacer() {
      return new SpacerMarker();
    },
    domEventHandlers: {
      mousedown(view, line) {
        const lineNo = view.state.doc.lineAt(line.from).number;
        view.dispatch({ effects: breakpointEffect.of(lineNo) });
        const bps = view.state.field(breakpointState);
        emit('breakpointsChange', Array.from(bps));
        emit('line-click', lineNo);
        return true;
      },
    },
  }),
];

// ============================================================================
// Debug Current Line Highlight
// ============================================================================

const debugLineEffect = StateEffect.define<number | null>();
const crossHighlightEffect = StateEffect.define<number | null>();

const debugLineState = StateField.define<DecorationSet>({
  create() { return Decoration.none; },
  update(deco, tr) {
    for (const e of tr.effects) {
      if (e.is(debugLineEffect)) {
        if (e.value === null || e.value <= 0) return Decoration.none;
        const line = tr.state.doc.line(e.value);
        return Decoration.set([
          Decoration.line({ class: 'cm-debug-current-line' }).range(line.from),
        ]);
      }
    }
    return deco.map(tr.changes);
  },
  provide: (f) => EditorView.decorations.from(f),
});

const crossHighlightState = StateField.define<DecorationSet>({
  create() { return Decoration.none; },
  update(deco, tr) {
    for (const e of tr.effects) {
      if (e.is(crossHighlightEffect)) {
        if (e.value === null || e.value <= 0) return Decoration.none;
        const line = tr.state.doc.line(e.value);
        return Decoration.set([
          Decoration.line({ class: 'cm-cross-highlight-line' }).range(line.from),
        ]);
      }
    }
    return deco.map(tr.changes);
  },
  provide: (f) => EditorView.decorations.from(f),
});

const debugLineHighlight = [
  debugLineState,
  crossHighlightState,
  EditorView.baseTheme({
    '.cm-debug-current-line': {
      backgroundColor: '#0e639c40',
      borderLeft: '3px solid #0e639c',
    },
    '.cm-cross-highlight-line': {
      backgroundColor: '#7b4a0e40',
      borderLeft: '3px solid #ff9d00',
    },
    '.cm-breakpoint-gutter': {
      width: '22px',
    },
    '.cm-breakpoint-gutter .cm-gutterElement': {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
    },

    '.cm-empty-circle-marker, .cm-breakpoint-marker': {
      marginTop: '1px',
    },
    '.cm-gutterElement:hover .cm-empty-circle-marker': {
      opacity: '1 !important',
    },
  }),
];

function getDebugExtensions(): Extension[] {
  return [...breakpointGutter, ...debugLineHighlight];
}

// ============================================================================
// Editor Setup
// ============================================================================

onMounted(() => {
  if (!editorContainer.value) return;

  const extensions: Extension[] = [
    lineNumbers(),
    highlightActiveLine(),
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
    autoLanguage,
    oneDark,
    EditorView.updateListener.of((update) => {
      if (update.docChanged && !props.readOnly) {
        emit('update:modelValue', update.state.doc.toString());
      }
    }),
      // line-click is handled by direct DOM listener in onMounted below
    debugCompartment.of(props.isDebugging ? getDebugExtensions() : []),
  ];

  if (props.readOnly) {
    extensions.push(EditorView.editable.of(false));
  }

  if (props.onRun) {
    extensions.push(keymap.of([{
      key: 'Ctrl-Enter',
      run: () => { props.onRun?.(); return true; }
    }]));
  }

  const state = EditorState.create({
    doc: props.modelValue,
    extensions,
  });

  editorView = new EditorView({
    state,
    parent: editorContainer.value,
  });

  // Hover line highlight
  let hoverLine = 0;
  editorContainer.value.addEventListener('mousemove', (event) => {
    if (!editorView) return;
    const pos = editorView.posAtCoords({ x: event.clientX, y: event.clientY });
    if (pos !== null) {
      const line = editorView.state.doc.lineAt(pos).number;
      if (line !== hoverLine) {
        hoverLine = line;
        emit('hover-line', line);
      }
    }
  });
  editorContainer.value.addEventListener('mouseleave', () => {
    hoverLine = 0;
    emit('hover-line-leave');
  });

  // Direct DOM listener for lineNumbers gutter click (more reliable than CodeMirror domEventHandlers)
  const lineNumbersGutter = editorContainer.value.querySelector('.cm-lineNumbers');
  if (lineNumbersGutter) {
    lineNumbersGutter.addEventListener('click', (event) => {
      const gutterCell = (event.target as HTMLElement).closest('.cm-gutterElement');
      if (gutterCell && gutterCell.textContent) {
        const parsed = parseInt(gutterCell.textContent.trim(), 10);
        if (!isNaN(parsed)) {
          emit('line-click', parsed);
        }
      }
    });
  }
});

watch(() => props.modelValue, (newVal) => {
  if (editorView && editorView.state.doc.toString() !== newVal) {
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: newVal },
    });
  }
});

watch(() => props.isDebugging, (debugging) => {
  if (!editorView) return;
  editorView.dispatch({
    effects: debugCompartment.reconfigure(debugging ? getDebugExtensions() : []),
  });
});

watch(() => props.currentDebugLine, (line) => {
  if (!editorView) return;
  editorView.dispatch({ effects: debugLineEffect.of(line ?? null) });
});

watch(() => props.highlightedSourceLine, (line) => {
  if (!editorView) return;
  editorView.dispatch({ effects: crossHighlightEffect.of(line ?? null) });
});

watch(() => props.breakpoints, (bps) => {
  if (!editorView) return;
  const current = editorView.state.field(breakpointState, false);
  if (!current) return;
  const currentArr = Array.from(current);
  const newArr = bps || [];
  const toAdd = newArr.filter((l) => !currentArr.includes(l));
  const toRemove = currentArr.filter((l) => !newArr.includes(l));
  if (toAdd.length === 0 && toRemove.length === 0) return;
  const effects = [
    ...toAdd.map((l) => breakpointEffect.of(l)),
    ...toRemove.map((l) => breakpointEffect.of(l)),
  ];
  editorView.dispatch({ effects });
}, { deep: true });

onUnmounted(() => {
  editorView?.destroy();
});
</script>

<style scoped>
.editor-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
}
.editor-container :deep(.cm-editor) {
  height: 100%;
  font-size: 14px;
}
.editor-container :deep(.cm-scroller) {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}
.editor-container :deep(.cm-gutters) {
  cursor: default;
}
.editor-container :deep(.cm-lineNumbers) {
  cursor: pointer;
}
.editor-container :deep(.cm-breakpoint-gutter) {
  cursor: pointer;
}
</style>
