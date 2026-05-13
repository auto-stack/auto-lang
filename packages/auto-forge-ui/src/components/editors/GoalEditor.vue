<template>
  <div class="goal-editor">
    <div class="form-row">
      <label>Title</label>
      <input v-model="draft.title" class="form-input" />
    </div>

    <div class="form-row">
      <label>Priority</label>
      <select v-model="draft.priority" class="form-select">
        <option value="">—</option>
        <option value="P0">P0</option>
        <option value="P1">P1</option>
        <option value="P2">P2</option>
      </select>
    </div>

    <div class="form-row">
      <label>Acceptance Criteria</label>
      <div class="criteria-list">
        <div
          v-for="(c, idx) in draft.criteria"
          :key="idx"
          class="criteria-row"
        >
          <input
            type="checkbox"
            v-model="c.checked"
            class="criteria-check"
          />
          <input
            v-model="c.text"
            class="criteria-input"
            placeholder="Testable criterion..."
          />
          <button class="criteria-remove" @click="removeCriterion(idx)">
            <X :size="12" />
          </button>
        </div>
        <button class="add-criterion" @click="addCriterion">
          <Plus :size="12" />
          Add criterion
        </button>
      </div>
    </div>

    <div class="form-row">
      <label>
        Details
        <span class="char-count" :class="{ over: detailsLength > 500 }">
          {{ detailsLength }}/500
        </span>
      </label>
      <textarea
        v-model="draft.details"
        class="form-textarea"
        rows="5"
        placeholder="≤500 words"
      />
    </div>

    <div class="form-row">
      <label>Depends on</label>
      <TagInput v-model="draft.depends_on" placeholder="G1, A1..." />
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
import { ref, computed } from 'vue'
import { parseGoalContent, serializeGoalForm } from '@/utils/goalParser'
import TagInput from './TagInput.vue'
import { Check, X, Plus } from 'lucide-vue-next'

const props = defineProps<{
  item: { title: string; content: string; priority?: string; depends_on?: string[] }
}>()

const emit = defineEmits<{
  save: [payload: { title: string; content: string; priority: string; depends_on: string[] }]
  cancel: []
}>()

const parsed = parseGoalContent(props.item.content, props.item.title)
parsed.priority = props.item.priority || ''
parsed.depends_on = props.item.depends_on ? [...props.item.depends_on] : []

const draft = ref(parsed)

const detailsLength = computed(() => draft.value.details.length)

function addCriterion() {
  draft.value.criteria.push({ text: '', checked: false })
}

function removeCriterion(idx: number) {
  draft.value.criteria.splice(idx, 1)
}

function onSave() {
  emit('save', {
    title: draft.value.title,
    content: serializeGoalForm(draft.value),
    priority: draft.value.priority,
    depends_on: draft.value.depends_on,
  })
}
</script>

<style scoped>
.goal-editor {
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
  display: flex;
  justify-content: space-between;
}

.char-count {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--af-muted);
}
.char-count.over {
  color: hsl(var(--destructive));
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

.form-textarea {
  resize: vertical;
  min-height: 100px;
}

.criteria-list {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.criteria-row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.criteria-check {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  cursor: pointer;
}

.criteria-input {
  flex: 1;
  padding: 0.3rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--background));
  color: var(--af-fg);
  font-size: 0.82rem;
  outline: none;
}

.criteria-remove {
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

.criteria-remove:hover {
  background: hsl(var(--destructive) / 0.1);
  color: hsl(var(--destructive));
}

.add-criterion {
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

.add-criterion:hover {
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
