<script setup lang="ts">
// 命中字符高亮。
//
// 用法：<Highlight :text="snippet.title" :query="searchQuery" />
//
// 规则：
//   - query 为空 / 仅空白 → 原样渲染 text。
//   - 否则 query 按空白拆分多关键字，每个大小写不敏感子串匹配，命中处包 <mark>。
//   - 在原始 text 上扫描位置，按片段分别转义 HTML，最后拼接——
//     既保证 HTML 注入安全，又能正确高亮包含 `&` `<` 等字符的命中。

import { computed } from 'vue'

const props = defineProps<{
  text: string
  query: string
}>()

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

const rendered = computed(() => {
  const text = props.text ?? ''
  const q = (props.query ?? '').trim()
  if (!q) return escapeHtml(text)

  const parts = q.split(/\s+/).filter(Boolean)
  if (parts.length === 0) return escapeHtml(text)

  // 长词优先匹配，避免短词截断长词。
  const pattern = new RegExp(
    '(' + parts.sort((a, b) => b.length - a.length).map(escapeRegExp).join('|') + ')',
    'gi'
  )

  // 在原始 text 上分段：未命中段做 escapeHtml；命中段做 escapeHtml 后包 <mark>。
  let out = ''
  let last = 0
  for (const m of text.matchAll(pattern)) {
    const idx = m.index ?? 0
    if (idx > last) out += escapeHtml(text.slice(last, idx))
    out += '<mark>' + escapeHtml(m[0]) + '</mark>'
    last = idx + m[0].length
  }
  if (last < text.length) out += escapeHtml(text.slice(last))
  return out
})
</script>

<template>
  <span v-html="rendered" />
</template>

<style scoped>
:deep(mark) {
  background: #fef3c7;
  color: #92400e;
  border-radius: 2px;
  padding: 0 1px;
}
</style>
