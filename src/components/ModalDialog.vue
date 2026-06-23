<script setup lang="ts">
// 通用模态对话框。
//
// 用法：
//   - 表单模式：传 fields，提交时 emit('submit', { fieldName: value })
//   - 确认模式：不传 fields，仅展示 message，确认时 emit('submit', {})
//
// 交互：Esc 取消、点击遮罩取消、回车提交（表单模式）、焦点自动聚焦首个输入。
// M3 的 SnippetEditor 会基于此组件扩展更复杂的表单。

import { ref, watch, nextTick, computed } from 'vue'

/** 单个表单字段的描述。 */
export interface FieldSchema {
  /** 字段名，submit 事件的 key 用此值。 */
  name: string
  /** 展示标签。 */
  label: string
  placeholder?: string
  /** 是否必填；必填且空时禁用提交按钮。 */
  required?: boolean
  /** 初始值。 */
  defaultValue?: string
}

const props = withDefaults(
  defineProps<{
    /** 是否显示。用 v-model:modelValue 绑定。 */
    modelValue: boolean
    title: string
    /** 表单字段。为空时进入「确认模式」，只展示 message。 */
    fields?: FieldSchema[]
    /** 确认模式下展示的正文。 */
    message?: string
    confirmText?: string
    cancelText?: string
    /** 确认按钮是否用危险色（删除场景）。 */
    danger?: boolean
  }>(),
  {
    fields: () => [],
    message: '',
    confirmText: '确定',
    cancelText: '取消',
    danger: false,
  },
)

const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
  (e: 'submit', values: Record<string, string>): void
  (e: 'cancel'): void
}>()

const values = ref<Record<string, string>>({})
/** 第一个字段的 input ref，用于自动聚焦。 */
const firstInputRef = ref<HTMLInputElement | null>(null)

// 打开时重置表单值。
watch(
  () => props.modelValue,
  async open => {
    if (open) {
      values.value = {}
      for (const f of props.fields) {
        values.value[f.name] = f.defaultValue ?? ''
      }
      await nextTick()
      firstInputRef.value?.focus()
    }
  },
)

const isConfirmMode = computed(() => props.fields.length === 0)

/** 必填字段是否都已填。 */
const canSubmit = computed(() => {
  if (isConfirmMode.value) return true
  return props.fields.every(
    f => !f.required || (values.value[f.name] ?? '').trim() !== '',
  )
})

function close(cancel: boolean) {
  emit('update:modelValue', false)
  if (cancel) emit('cancel')
}

function onSubmit() {
  if (!canSubmit.value) return
  emit('submit', { ...values.value })
  emit('update:modelValue', false)
}

function onBackdropClick() {
  close(true)
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    close(true)
  } else if (e.key === 'Enter' && !isConfirmMode.value) {
    // 表单模式回车提交。
    e.preventDefault()
    onSubmit()
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="backdrop"
      @mousedown.self="onBackdropClick"
      @keydown="onKeydown"
    >
      <div
        class="modal"
        :class="{ danger }"
        role="dialog"
        aria-modal="true"
        :aria-label="title"
      >
        <header class="modal-head">
          <h2>{{ title }}</h2>
        </header>

        <div class="modal-body">
          <p v-if="isConfirmMode" class="confirm-msg">{{ message }}</p>
          <div v-else class="form">
            <label v-for="(f, i) in fields" :key="f.name" class="field">
              <span class="label">{{ f.label }}<span v-if="f.required" class="req">*</span></span>
              <input
                :ref="i === 0 ? 'firstInputRef' : undefined"
                v-model="values[f.name]"
                type="text"
                :placeholder="f.placeholder"
                @keydown.enter.prevent="onSubmit"
              />
            </label>
          </div>
        </div>

        <footer class="modal-foot">
          <button class="btn btn-cancel" @click="close(true)">{{ cancelText }}</button>
          <button
            class="btn btn-confirm"
            :class="{ danger }"
            :disabled="!canSubmit"
            @click="onSubmit"
          >
            {{ confirmText }}
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.modal {
  width: 360px;
  max-width: calc(100vw - 32px);
  background: var(--bg-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}
.modal-head {
  padding: var(--space-4) var(--space-4) var(--space-2);
}
.modal-head h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}
.modal-body {
  padding: var(--space-2) var(--space-4);
}
.confirm-msg {
  margin: 0;
  color: var(--fg-1);
  line-height: 1.5;
  font-size: 13px;
}
.form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.label {
  font-size: 12px;
  color: var(--fg-2);
}
.req {
  color: var(--danger);
  margin-left: 2px;
}
.field input {
  padding: var(--space-2) var(--space-3);
  background: var(--bg-3);
  color: var(--fg-1);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  font: inherit;
  outline: none;
}
.field input:focus {
  border-color: var(--accent);
}
.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4) var(--space-4);
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
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.btn-confirm {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.btn-confirm.danger {
  background: var(--danger);
  border-color: var(--danger);
}
.btn-confirm:hover:not(:disabled) {
  filter: brightness(1.1);
}
</style>
