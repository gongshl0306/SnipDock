<script setup lang="ts">
// 设置弹窗：
//   1. 录制全局呼出快捷键
//   2. 数据导出 / 导入（JSON 备份）
//
// 录制流程：点「录制」→ 监听 keydown → 要求至少一个修饰键（Ctrl/Cmd/Alt/Shift）
//   + 一个普通键 → 显示组合（如 "Ctrl+Shift+D"）。
// 不接受单修饰键或纯字母（避免误触发）。
// 「清除」= 设为空（不注册任何快捷键）。
// 「保存」调 set_toggle_shortcut，后端做注册 + 持久化；
// 失败（格式错/被占用）显示中文错误。
//
// 导出：弹保存对话框，把全部分类+片段写成 JSON。
// 导入：弹打开对话框，合并到当前库（分类按 name、片段按三元组去重）。
//       导入成功后重新加载内存中的 categories / snippets。

import { ref, computed } from 'vue'
import { getSettings, setToggleShortcut } from '@/api/settings'
import { exportData, importData } from '@/api/backup'
import { useCategories } from '@/composables/useCategories'
import { useSnippets } from '@/composables/useSnippets'
import { parseAppError } from '@/types/models'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
}>()

/** 后端已保存的快捷键（打开时从后端拉）。 */
const savedShortcut = ref<string | null>(null)
/** 录制中的候选值（未保存）。null = 未录制。 */
const recorded = ref<string | null>(null)
const isRecording = ref(false)
const errorMsg = ref('')
const saving = ref(false)
const loaded = ref(false)

/** 导出 / 导入 状态。null = 空闲。 */
const backupBusy = ref<'export' | 'import' | null>(null)
/** 导出/导入成功后的提示文字（蓝色）。 */
const backupInfo = ref('')

const { load: reloadCategories } = useCategories()
const { load: reloadSnippets } = useSnippets()

/** 当前显示的快捷键：优先录制中的，否则已保存的。 */
const displayShortcut = computed(() => recorded.value ?? savedShortcut.value)

/** 显示文本：无快捷键时给提示。 */
const displayText = computed(() => {
  const v = displayShortcut.value
  return v && v.length > 0 ? v : '（未设置）'
})

async function load() {
  if (loaded.value) return
  loaded.value = true
  try {
    const s = await getSettings()
    savedShortcut.value = s.toggle_shortcut
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  }
}

// 每次打开都重新加载（防外部改动）。
import { watch } from 'vue'
watch(
  () => props.modelValue,
  open => {
    if (open) {
      loaded.value = false
      recorded.value = null
      errorMsg.value = ''
      backupInfo.value = ''
      backupBusy.value = null
      isRecording.value = false
      void load()
    }
  },
  { immediate: true },
)

/** 把键盘事件转成 Tauri accelerator 字符串。返回 null 表示不合法。 */
function eventToAccelerator(e: KeyboardEvent): string | null {
  const mods: string[] = []
  if (e.ctrlKey) mods.push('Ctrl')
  if (e.altKey) mods.push('Alt')
  if (e.shiftKey) mods.push('Shift')
  if (e.metaKey) mods.push('Super')

  // 必须有至少一个修饰键。
  if (mods.length === 0) return null

  // 忽略单独按修饰键的情况。
  const key = e.key
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(key)) return null

  // 把 key 规范化：字母大写、特殊键映射。
  let keyName: string
  if (key === ' ') keyName = 'Space'
  else if (key.length === 1) keyName = key.toUpperCase()
  else keyName = key // F1, ArrowUp, Enter 等

  return [...mods, keyName].join('+')
}

function onKeydown(e: KeyboardEvent) {
  if (!isRecording.value) return
  e.preventDefault()
  e.stopPropagation()
  const accel = eventToAccelerator(e)
  if (accel) {
    recorded.value = accel
    isRecording.value = false
    errorMsg.value = ''
  }
  // 不合法的组合（纯修饰键、无修饰键）静默忽略，等用户继续按。
}

function startRecording() {
  recorded.value = null
  isRecording.value = true
  errorMsg.value = ''
}

function clearShortcut() {
  recorded.value = '' // 空串表示清除
  isRecording.value = false
  errorMsg.value = ''
}

async function save() {
  saving.value = true
  errorMsg.value = ''
  try {
    // 录制中或没动过都按「当前显示值」存；
    // displayShortcut 为 null/'' 都会清除。
    const accel = recorded.value !== null ? recorded.value : (savedShortcut.value ?? '')
    await setToggleShortcut(accel)
    savedShortcut.value = accel.length > 0 ? accel : null
    recorded.value = null
    emit('update:modelValue', false)
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  } finally {
    saving.value = false
  }
}

function close() {
  isRecording.value = false
  emit('update:modelValue', false)
}

function onBackdropClick() {
  close()
}

async function onExport() {
  if (backupBusy.value) return
  backupBusy.value = 'export'
  errorMsg.value = ''
  backupInfo.value = ''
  try {
    const r = await exportData()
    if (r) {
      backupInfo.value = `已导出 ${r.categories} 个分类 / ${r.snippets} 条片段 → ${r.path}`
    }
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  } finally {
    backupBusy.value = null
  }
}

async function onImport() {
  if (backupBusy.value) return
  backupBusy.value = 'import'
  errorMsg.value = ''
  backupInfo.value = ''
  try {
    const r = await importData()
    if (r) {
      // 合并完成后重新加载主窗口数据。
      await reloadCategories()
      await reloadSnippets()
      backupInfo.value =
        `导入完成：新建分类 ${r.created_categories}、复用 ${r.reused_categories}；` +
        `新增片段 ${r.created_snippets}、跳过重复 ${r.skipped_snippets}`
    }
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  } finally {
    backupBusy.value = null
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="backdrop"
      tabindex="0"
      @mousedown.self="onBackdropClick"
      @keydown="onKeydown"
    >
      <div class="modal" role="dialog" aria-modal="true" aria-label="设置">
        <header class="modal-head">
          <h2>设置</h2>
        </header>

        <div class="modal-body">
          <div class="setting-row">
            <div class="setting-label">
              <div class="title">呼出快捷键</div>
              <div class="desc">显示/隐藏主窗口的全局快捷键</div>
            </div>
            <div class="setting-control">
              <span class="kbd-display">{{ isRecording ? '按下组合键…' : displayText }}</span>
              <button
                class="btn small"
                :class="{ recording: isRecording }"
                @click="startRecording"
              >
                {{ isRecording ? '录制中…' : '录制' }}
              </button>
              <button
                v-if="displayShortcut && displayShortcut.length > 0"
                class="btn small"
                @click="clearShortcut"
              >
                清除
              </button>
            </div>
          </div>

          <hr class="divider" />

          <div class="setting-row">
            <div class="setting-label">
              <div class="title">数据备份</div>
              <div class="desc">导出全部分类与片段为 JSON；导入时按名称去重合并，不会清空当前数据</div>
            </div>
            <div class="setting-control">
              <button class="btn small" :disabled="!!backupBusy" @click="onExport">
                {{ backupBusy === 'export' ? '导出中…' : '导出' }}
              </button>
              <button class="btn small" :disabled="!!backupBusy" @click="onImport">
                {{ backupBusy === 'import' ? '导入中…' : '导入' }}
              </button>
            </div>
          </div>

          <p v-if="backupInfo" class="info">{{ backupInfo }}</p>
          <p v-if="errorMsg" class="err">{{ errorMsg }}</p>
        </div>

        <footer class="modal-foot">
          <button class="btn" @click="close">取消</button>
          <button class="btn primary" :disabled="saving || isRecording" @click="save">
            {{ saving ? '保存中…' : '保存' }}
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
  outline: none;
}
.modal {
  width: 480px;
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
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}
.setting-label .title {
  font-size: 13px;
  color: var(--fg-1);
}
.setting-label .desc {
  font-size: 11px;
  color: var(--fg-2);
  margin-top: 2px;
}
.setting-control {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.kbd-display {
  min-width: 120px;
  padding: var(--space-2) var(--space-3);
  background: var(--bg-1);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  font-family: var(--mono);
  font-size: 12px;
  color: var(--fg-1);
  text-align: center;
}
.err {
  margin: var(--space-3) 0 0;
  color: var(--danger);
  font-size: 12px;
}
.info {
  margin: var(--space-3) 0 0;
  color: var(--accent);
  font-size: 12px;
  line-height: 1.4;
  word-break: break-all;
}
.divider {
  border: none;
  border-top: 1px solid var(--border);
  margin: var(--space-3) 0;
}
.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4) var(--space-4);
}
.btn {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  background: var(--bg-3);
  color: var(--fg-1);
  cursor: pointer;
  font: inherit;
}
.btn.small {
  padding: var(--space-1) var(--space-2);
  font-size: 12px;
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
.btn.recording {
  border-color: var(--accent);
  color: var(--accent);
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
