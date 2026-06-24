<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Setting } from '@element-plus/icons-vue'
import CategoryList from '@/components/CategoryList.vue'
import SnippetList from '@/components/SnippetList.vue'
import SnippetDetail from '@/components/SnippetDetail.vue'
import SnippetEditor from '@/components/SnippetEditor.vue'
import SettingsDialog from '@/components/SettingsDialog.vue'
import { useCategories } from '@/composables/useCategories'
import { useSnippets } from '@/composables/useSnippets'
import { bindShortcuts } from '@/composables/useShortcuts'

const { load: loadCategories } = useCategories()
const { isEditing, load: loadSnippets, bindCategoryWatcher, searchQuery } = useSnippets()

// 设置弹窗可见性。
const settingsOpen = ref(false)

onMounted(async () => {
  // 加载分类，再加载片段；并注册「分类切换→自动重载」watch。
  await loadCategories()
  bindCategoryWatcher()
  await loadSnippets()
  bindShortcuts()
})
</script>

<template>
  <main class="shell">
    <header class="topbar">
      <input
        v-model="searchQuery"
        class="search"
        type="text"
        placeholder="搜索片段…"
      />
      <button
        class="settings-btn"
        title="设置"
        @click="settingsOpen = true"
      ><Setting class="icon" /></button>
    </header>
    <section class="cols">
      <aside class="col col-cat">
        <CategoryList />
      </aside>
      <section class="col col-list">
        <SnippetList />
      </section>
      <section class="col col-detail">
        <SnippetEditor v-if="isEditing" />
        <SnippetDetail v-else />
      </section>
    </section>
    <SettingsDialog v-model="settingsOpen" />
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
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-2);
}
.search {
  flex: 1 1 auto;
  width: auto;
  padding: 8px 12px;
  background: var(--bg-3);
  color: var(--fg-1);
  border: 1px solid var(--border);
  border-radius: 6px;
  font: inherit;
}
.settings-btn {
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-3);
  color: var(--fg-2);
  cursor: pointer;
  font-size: 15px;
  line-height: 1;
}
.settings-btn:hover {
  border-color: var(--fg-2);
  color: var(--fg-1);
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
</style>
