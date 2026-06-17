<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// M1：仅做启动验证。后续里程碑会接 useCategories / useSnippets。
const status = ref<'loading' | 'ok' | 'err'>('loading')
const message = ref('')

onMounted(async () => {
  try {
    const reply = await invoke<string>('ping')
    message.value = reply
    status.value = 'ok'
  } catch (e: unknown) {
    message.value = String(e)
    status.value = 'err'
  }
})
</script>

<template>
  <main class="shell">
    <header class="topbar">
      <input
        class="search"
        placeholder="Search snippets, commands, tags..."
        disabled
      />
    </header>
    <section class="cols">
      <aside class="col col-cat">
        <div class="placeholder">分类（M2）</div>
      </aside>
      <section class="col col-list">
        <div class="placeholder">片段列表（M3）</div>
      </section>
      <section class="col col-detail">
        <div class="placeholder">片段详情（M3）</div>
      </section>
    </section>
    <footer class="statusbar">
      <span v-if="status === 'loading'">连接后端中…</span>
      <span v-else-if="status === 'ok'" class="ok">{{ message }}</span>
      <span v-else class="err">后端连接失败：{{ message }}</span>
    </footer>
  </main>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-1);
  color: var(--fg-1);
}
.topbar {
  flex: 0 0 auto;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-2);
}
.search {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-3);
  color: var(--fg-1);
  border: 1px solid var(--border);
  border-radius: 6px;
  font: inherit;
}
.cols {
  flex: 1 1 auto;
  display: grid;
  grid-template-columns: 18% 32% 1fr;
  min-height: 0;
}
.col {
  border-right: 1px solid var(--border);
  overflow: auto;
}
.col-detail {
  border-right: none;
}
.placeholder {
  padding: 16px;
  color: var(--fg-2);
  font-size: 13px;
}
.statusbar {
  flex: 0 0 auto;
  padding: 6px 12px;
  border-top: 1px solid var(--border);
  background: var(--bg-2);
  font-size: 12px;
  color: var(--fg-2);
}
.statusbar .ok { color: var(--accent); }
.statusbar .err { color: #e57373; }
</style>
