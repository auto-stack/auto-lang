<template>
  <div ref="editorContainer" class="editor-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from 'vue';
import { EditorState, type Extension } from '@codemirror/state';
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view';
import { defaultKeymap, indentWithTab, history, historyKeymap } from '@codemirror/commands';
import { oneDark } from '@codemirror/theme-one-dark';
import { autoLanguage } from '../lang/auto';

const props = defineProps<{
  modelValue: string;
  onRun?: () => void;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'line-click': [line: number];
}>();

const editorContainer = ref<HTMLDivElement>();
let editorView: EditorView | null = null;

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
      if (update.docChanged) {
        emit('update:modelValue', update.state.doc.toString());
      }
    }),
    EditorView.domEventHandlers({
      mousedown: (event, view) => {
        // Check if the click is in the gutter area
        const target = event.target as HTMLElement;
        if (target.closest('.cm-gutters') || target.closest('.cm-gutter')) {
          const pos = view.posAtCoords({ x: event.clientX, y: event.clientY }, false);
          if (pos !== null) {
            const line = view.state.doc.lineAt(pos);
            emit('line-click', line.number);
            return true;
          }
        }
        return false;
      },
    }),
  ];

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
});

watch(() => props.modelValue, (newVal) => {
  if (editorView && editorView.state.doc.toString() !== newVal) {
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: newVal },
    });
  }
});

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
  cursor: pointer;
}
</style>
