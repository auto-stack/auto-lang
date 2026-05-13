<template>
  <div class="test-editor">
    <div class="form-row">
      <label>Title</label>
      <input v-model="draft.title" class="form-input" />
    </div>

    <div class="form-row">
      <label>Type</label>
      <select v-model="draft.type" class="form-select">
        <option value="Unit">Unit</option>
        <option value="Integration">Integration</option>
        <option value="E2E">E2E</option>
        <option value="Contract">Contract</option>
        <option value="Performance">Performance</option>
        <option value="Fuzz">Fuzz</option>
      </select>
    </div>

    <div class="form-row">
      <label>Scope / Parent Goal</label>
      <input v-model="draft.scope" class="form-input" placeholder="G1" />
    </div>

    <div class="form-row">
      <label>Fixture</label>
      <textarea v-model="draft.fixture" class="form-textarea code" rows="4" />
    </div>

    <div class="form-row">
      <label>Steps</label>
      <div class="steps-list">
        <div v-for="(step, idx) in draft.steps" :key="idx" class="step-row">
          <span class="step-num">{{ idx + 1 }}.</span>
          <input v-model="draft.steps[idx]" class="step-input" />
          <button class="step-remove" @click="removeStep(idx)">
            <X :size="12" />
          </button>
        </div>
        <button class="add-step" @click="addStep">
          <Plus :size="12" />
          Add step
        </button>
      </div>
    </div>

    <div class="form-row">
      <label>Expected Outcome</label>
      <textarea v-model="draft.expected" class="form-textarea" rows="3" />
    </div>

    <div class="form-row">
      <label>Test File Path</label>
      <input v-model="draft.testFile" class="form-input monospace" placeholder="tests/foo.rs" />
    </div>

    <div class="editor-actions">
      <button class="save-btn" @click="onSave">
        <Check :size="13" />
        Save
      </button>
      <button class="cancel-btn" @click="$emit('cancel')">
        <X :size="13" />
        Cancel
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Check, X, Plus } from 'lucide-vue-next'

const props = defineProps<{
  item: { title: string; content: string; test_file?: string }
}>()

const emit = defineEmits<{
  save: [payload: { title: string; content: string; test_file: string }]
  cancel: []
}>()

const c = props.item.content
const draft = ref({
  title: props.item.title,
  type: (c.match(/\*\*Type:\*\*\s*(\w+)/)?.[1] || 'Unit'),
  scope: (c.match(/\*\*Scope:\*\*\s*(.+)/)?.[1] || ''),
  fixture: extractBlock(c, 'Fixture') || '',
  steps: extractSteps(c),
  expected: extractBlock(c, 'Expected Outcome') || '',
  testFile: props.item.test_file || '',
})

function extractBlock(content: string, label: string): string {
  const re = new RegExp(`\\*\\*${label}:\\*\\*\\s*\\n?([\\s\\S]*?)(?=\\n\\*\\*|$)`, 'i')
  const m = content.match(re)
  return m ? m[1].trim() : ''
}

function extractSteps(content: string): string[] {
  const steps: string[] = []
  const re = /^\d+\.\s*(.*)$/gm
  let m: RegExpExecArray | null
  while ((m = re.exec(content)) !== null) {
    steps.push(m[1].trim())
  }
  return steps.length ? steps : ['']
}

function addStep() {
  draft.value.steps.push('')
}

function removeStep(idx: number) {
  draft.value.steps.splice(idx, 1)
}

function onSave() {
  const lines: string[] = []
  if (draft.value.type) lines.push(`**Type:** ${draft.value.type}`)
  if (draft.value.scope) lines.push(`**Scope:** ${draft.value.scope}`)
  if (draft.value.fixture) {
    lines.push('**Fixture:**')
    lines.push('```')
    lines.push(draft.value.fixture)
    lines.push('```')
  }
  if (draft.value.steps.length > 0 && draft.value.steps.some(s => s.trim())) {
    lines.push('')
    draft.value.steps.forEach((s, i) => {
      if (s.trim()) lines.push(`${i + 1}. ${s.trim()}`)
    })
  }
  if (draft.value.expected) {
    lines.push('')
    lines.push('**Expected Outcome:**')
    lines.push(draft.value.expected)
  }

  emit('save', {
    title: draft.value.title,
    content: lines.join('\n'),
    test_file: draft.value.testFile,
  })
}
</script>

<style scoped>
.test-editor {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.form-row label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--af-muted);
}

.form-input,
.form-select,
.form-textarea {
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--background));
  color: var(--af-fg);
  font-size: 0.85rem;
  outline: none;
}

.form-input:focus,
.form-select:focus,
.form-textarea:focus {
  border-color: hsl(var(--primary) / 0.5);
}

.form-textarea.code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.8rem;
}

.form-input.monospace {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.steps-list {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.step-row {
  display: flex;
  align-items: center;
  gap: 0.3rem;
}

.step-num {
  font-size: 0.8rem;
  color: var(--af-muted);
  min-width: 1.2rem;
}

.step-input {
  flex: 1;
  padding: 0.3rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--background));
  color: var(--af-fg);
  font-size: 0.82rem;
  outline: none;
}

.step-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
}

.step-remove:hover {
  background: hsl(var(--destructive) / 0.1);
  color: hsl(var(--destructive));
}

.add-step {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.3rem 0.5rem;
  font-size: 0.75rem;
  border-radius: 6px;
  border: 1px dashed var(--af-border);
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
  align-self: flex-start;
}

.add-step:hover {
  color: hsl(var(--primary));
  border-color: hsl(var(--primary) / 0.4);
}

.editor-actions {
  display: flex;
  gap: 0.5rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--af-border);
}

.save-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.4rem 0.8rem;
  font-size: 0.78rem;
  font-weight: 600;
  border-radius: 6px;
  border: none;
  background: hsl(var(--primary));
  color: white;
  cursor: pointer;
}

.save-btn:hover {
  opacity: 0.9;
}

.cancel-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.4rem 0.8rem;
  font-size: 0.78rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
}

.cancel-btn:hover {
  color: var(--af-fg);
  background: hsl(var(--muted-foreground) / 0.05);
}
</style>
