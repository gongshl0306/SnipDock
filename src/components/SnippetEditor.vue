<script setup lang="ts">
// 右栏（编辑态）：新增 / 编辑片段表单。
//
// 编辑模式由 editingSnippet 是否为 null 区分。
// 分类默认值：编辑模式用原片段；新增模式用 selectedCategoryId ?? 第一个分类。
// 保存：调 create/update，成功后 endEdit。后端做 trim/校验，前端禁用按钮兜底。

import { ref, computed, watch, nextTick } from 'vue'
import { useSnippets } from '@/composables/useSnippets'
import { useCategories } from '@/composables/useCategories'
import { parseAppError } from '@/types/models'
import type { CreateSnippetPayload, UpdateSnippetPayload } from '@/types/models'

const {
  editingSnippet,
  create,
  update,
  cancelEdit,
  endEdit,
} = useSnippets()
const { categories, selectedCategoryId } = useCategories()

interface FormState {
  category_id: number
  title: string
  content: string
  favorite: boolean
}

const form = ref<FormState>(makeEmptyForm())
const errorMsg = ref('')
const saving = ref(false)
const titleInputRef = ref<HTMLInputElement | null>(null)

function makeEmptyForm(): FormState {
  return {
    category_id: 0,
    title: '',
    content: '',
    favorite: false,
  }
}

/** 每次进入编辑模式时初始化表单。 */
function initForm() {
  if (editingSnippet.value) {
    // 编辑模式：用原片段。
    const s = editingSnippet.value
    form.value = {
      category_id: s.category_id,
      title: s.title,
      content: s.content,
      favorite: s.favorite !== 0,
    }
  } else {
    // 新增模式：预选当前分类（或第一个）。
    const initialCat =
      selectedCategoryId.value ?? categories.value[0]?.id ?? 0
    form.value = { ...makeEmptyForm(), category_id: initialCat }
  }
  errorMsg.value = ''
  nextTick(() => titleInputRef.value?.focus())
}

// 组件挂载 + editingSnippet 变化都触发初始化。
initForm()
watch(editingSnippet, initForm)

const isEdit = computed(() => editingSnippet.value !== null)
const canSubmit = computed(
  () =>
    !saving.value &&
    form.value.category_id !== 0 &&
    form.value.title.trim() !== '' &&
    form.value.content.trim() !== '',
)

async function onSave() {
  if (!canSubmit.value) return
  saving.value = true
  errorMsg.value = ''
  try {
    const fav = form.value.favorite ? 1 : 0
    if (isEdit.value && editingSnippet.value) {
      const payload: UpdateSnippetPayload = {
        id: editingSnippet.value.id,
        category_id: form.value.category_id,
        title: form.value.title,
        content: form.value.content,
        favorite: fav,
      }
      await update(payload)
    } else {
      const payload: CreateSnippetPayload = {
        category_id: form.value.category_id,
        title: form.value.title,
        content: form.value.content,
        favorite: fav,
      }
      await create(payload)
    }
    endEdit()
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  } finally {
    saving.value = false
  }
}

function onCancel() {
  cancelEdit()
}
</script>

<template>
  <div class="editor-panel">
    <div class="editor-head">
      <h2>{{ isEdit ? '编辑片段' : '新增片段' }}</h2>
    </div>

    <div class="editor-scroll">
      <label class="field">
        <span class="label">分类<span class="req">*</span></span>
        <select v-model.number="form.category_id">
          <option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option>
        </select>
      </label>

      <label class="field">
        <span class="label">标题<span class="req">*</span></span>
        <input
          ref="titleInputRef"
          v-model="form.title"
          type="text"
          placeholder="查看所有 Pod"
        />
      </label>

      <label class="field">
        <span class="label">内容<span class="req">*</span></span>
        <textarea
          v-model="form.content"
          rows="8"
          placeholder="kubectl get pods -A -o wide"
          class="mono"
        />
      </label>

      <label class="field check">
        <input v-model="form.favorite" type="checkbox" />
        <span>收藏</span>
      </label>
    </div>

    <div v-if="errorMsg" class="err-msg">{{ errorMsg }}</div>

    <div class="actions">
      <button class="btn" @click="onCancel">取消 (Esc)</button>
      <button class="btn primary" :disabled="!canSubmit" @click="onSave">
        {{ saving ? '保存中…' : '保存' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-2);
}
.editor-head {
  flex: 0 0 auto;
  padding: var(--space-4);
  border-bottom: 1px solid var(--border);
}
.editor-head h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}
.editor-scroll {
  flex: 1 1 auto;
  overflow: auto;
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.field .label {
  font-size: 12px;
  color: var(--fg-2);
}
.req {
  color: var(--danger);
  margin-left: 2px;
}
.field input[type="text"],
.field select,
.field textarea {
  padding: var(--space-2) var(--space-3);
  background: var(--bg-3);
  color: var(--fg-1);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  font: inherit;
  outline: none;
  width: 100%;
}
.field textarea {
  resize: vertical;
  min-height: 120px;
  line-height: 1.5;
}
.field .mono {
  font-family: var(--mono);
  font-size: 13px;
}
.field input:focus,
.field select:focus,
.field textarea:focus {
  border-color: var(--accent);
}
.field.check {
  flex-direction: row;
  align-items: center;
  gap: var(--space-2);
}
.field.check input {
  width: auto;
}
.field.check span {
  font-size: 13px;
  color: var(--fg-1);
}
.err-msg {
  flex: 0 0 auto;
  padding: var(--space-2) var(--space-4);
  color: var(--danger);
  font-size: 12px;
  border-top: 1px solid var(--border);
}
.actions {
  flex: 0 0 auto;
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border);
}
.btn {
  padding: var(--space-2) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  background: var(--bg-3);
  color: var(--fg-1);
  cursor: pointer;
  font: inherit;
}
.btn:hover {
  border-color: var(--fg-2);
}
.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.btn.primary:hover:not(:disabled) {
  filter: brightness(1.1);
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>