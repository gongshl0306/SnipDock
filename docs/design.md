# SnipDock 第一版设计文档（v1）

> 本文档基于 [target.md](target.md) 中的产品规格，给出第一版的工程实现设计。
> **target.md 是需求与契约的源头**：字段名、命令名、表结构、键位均以其为准。
> 本文档关注**怎么做**，不复述**做什么**。

---

## 1. 范围与目标

第一版交付一个本地桌面端的代码片段管理器：分类组织、全局/分类内关键字搜索、键盘驱动复制。
**性能假设（v1）：** 总片段数 < 5000，本地单用户单进程运行，首次加载全量到内存即可。

### 不做的事（呼应 target.md §15）

为避免过度设计，下列特性 v1 一律不实现：

- 独立的 `tags` 表 / `snippet_tags` 关联表（v1 连字符串 `tags` 也不做）
- SQL 端搜索（FTS5、`LIKE` 联合查询）
- 多级分类 / 分类树
- 高级查询语法（`tag:foo`、`category:bar`）
- 多个示例分类（v1 仅创建 `默认`）
- 删除非空分类时级联或迁移片段（直接报错 `Category is not empty`）
- 系统托盘之外的其他系统集成（自动启动、通知等）
- 多窗口、多语言

---

## 2. 技术栈

| 层 | 选型 | 理由 |
|---|---|---|
| 应用壳 | Tauri 2.x | 原生窗口、SQLite 友好、产物体积小 |
| 前端 | Vue 3 + Vite + TypeScript | 组合式 API 简洁，spec §6 已用 TS 类型 |
| 状态管理 | Composables（`ref`/`computed`） | spec §9.2 已是 `ref`，v1 数据规模无需 Pinia |
| 后端 | Rust + tauri::command | spec §7 已规定 command 接口 |
| 数据库 | SQLite + rusqlite | 同步 API、小工具友好；连接以 `Mutex<Connection>` 管理（无需连接池） |
| 剪贴板 | 前端 `navigator.clipboard.writeText` | v1 简化方案，见 §6.4 |
| 全局快捷键 | `tauri-plugin-global-shortcut` | 实现可配置的呼出快捷键（默认未注册，用户在设置里录制） |
| 样式 | 原生 CSS（CSS Variables 主题） | UI 复杂度低，v1 不引入 Tailwind/UnoCSS |

> 这些选型只在与 spec 不冲突时生效。如果实现时发现某项选择导致与 spec §5–§7 的契约必须变形，先反馈，再调整。

---

## 3. 仓库目录结构

```
SnipDock/
├── docs/
│   ├── target.md              # 产品 spec（源头）
│   └── design.md              # 本文档
├── src/                       # 前端源码（Vue + TS）
│   ├── components/
│   │   ├── CategoryList.vue   # 左栏：分类列表
│   │   ├── SnippetList.vue    # 中栏：片段列表
│   │   ├── SnippetDetail.vue  # 右栏：详情视图
│   │   ├── SnippetEditor.vue  # 右栏：新增/编辑表单
│   │   └── SearchBar.vue      # 顶部搜索框
│   ├── composables/
│   │   ├── useCategories.ts   # 分类状态 + CRUD 包装
│   │   ├── useSnippets.ts     # 片段状态 + CRUD + 搜索/排序
│   │   ├── useShortcuts.ts    # 全局/局部快捷键绑定
│   │   └── useClipboard.ts    # 复制 + mark_used 调用
│   ├── api/
│   │   ├── categories.ts      # invoke 'list_categories' 等
│   │   └── snippets.ts        # invoke 'list_snippets' 等
│   ├── types/
│   │   └── models.ts          # Category / Snippet / Payload 类型（按 spec §6）
│   ├── styles/
│   │   ├── tokens.css         # CSS 变量（颜色、字号、间距）
│   │   └── global.css
│   ├── App.vue
│   └── main.ts
├── src-tauri/                 # Rust 端
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── categories.rs  # list/create/update/delete
│   │   │   └── snippets.rs    # list/create/update/delete/mark_used
│   │   ├── db/
│   │   │   ├── mod.rs         # Connection 管理 + State
│   │   │   ├── migration.rs   # 建表 + 索引 + 默认分类
│   │   │   └── repo.rs        # SQL 封装
│   │   └── error.rs           # AppError + serde::Serialize
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── README.md
└── CLAUDE.md
```

理由：把 SQL 集中在 `db/repo.rs` 的好处是，将来如果要替换为 sqlx 或加 FTS5，只改这一层。

---

## 4. 数据库层

### 4.1 表结构与索引

完全按照 [target.md](target.md) §5.1–§5.4。**不增减字段、不改名**：
- `categories(id, name, description, sort_order, created_at, updated_at)`
- `snippets(id, category_id NOT NULL FK, title, content, favorite, used_count, created_at, updated_at, last_used_at)`
- 索引：`idx_snippets_category_id` / `idx_snippets_updated_at` / `idx_snippets_used_count` / `idx_snippets_last_used_at`

### 4.2 数据库文件位置

通过 `tauri::api::path::app_data_dir(&config)` 解析，文件名 `snipdock.db`。
开发期可在 `tauri.conf.json` 中配置一个 `productName`，让 macOS / Windows / Linux 都落到约定目录。

### 4.3 启动迁移流程

`main.rs` 启动时执行（顺序敏感）：

1. 解析数据目录，确保父目录存在；
2. `Connection::open(path)`；
3. `PRAGMA foreign_keys = ON`；
4. 顺序执行 `CREATE TABLE IF NOT EXISTS …` × 2；
5. 顺序执行 `CREATE INDEX IF NOT EXISTS …` × 4；
6. `SELECT count(*) FROM categories`：若为 0，插入 `'默认'`；
7. 把 `Mutex<Connection>` 注入 Tauri `State` 供 commands 共享。

> **不要在 v1 引入 schema 版本号**。表结构未来变更时再加 `user_version` PRAGMA。

### 4.4 时间戳

- 写入：Rust 端用 `chrono::Utc::now().to_rfc3339()`，前端不传时间字段；
- 显示：前端按需格式化（spec 没要求本地化时区，v1 直接显示 RFC3339 截断到分钟）。

---

## 5. 后端 Tauri Commands

### 5.1 命令清单（按 spec §7）

```
list_categories
create_category   { name, description? }
update_category   { id, name, description? }
delete_category   { id }
list_snippets
list_snippets_by_category   { category_id }
create_snippet    { CreateSnippetPayload }
update_snippet    { UpdateSnippetPayload }
delete_snippet    { id }
mark_snippet_used { id }
```

### 5.2 v1 推迟实现的命令

下列 spec 列出的命令 v1 **不实现**（YAGNI），保留命名以便后续补齐：

- `search_snippets`：spec §9.1 已经说"前端内存中完成搜索"，v1 前端走 §9.4 的 `computed` 分支。后端命令空着即可。
- `copy_snippet`：v1 用前端 `navigator.clipboard.writeText` + `mark_snippet_used`，更轻。

> 这两个偏差**是设计决策，不是疏漏**。在实现到这里之前，请先和用户确认：是否同意这两条偏差？或希望从一开始就把后端版本一起做掉？

### 5.3 错误处理

```rust
// src-tauri/src/error.rs
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Category is not empty")]
    CategoryNotEmpty,
    #[error("Category name cannot be empty")]
    CategoryNameEmpty,
    #[error("Snippet title cannot be empty")]
    SnippetTitleEmpty,
    #[error("Snippet content cannot be empty")]
    SnippetContentEmpty,
    #[error("Not found: {kind} id={id}")]
    NotFound { kind: &'static str, id: i64 },
    #[error("Database error: {0}")]
    Db(String),       // 包装 rusqlite::Error 的 message
}
```

约定：
- `delete_category` 在分类下还有片段时返回 `CategoryNotEmpty`（spec §7.3 要求的字面量错误信息）；
- 所有命令返回 `Result<T, AppError>`；
- 前端在 `try/catch` 中按 `error.message` 国际化文案（v1 直接显示中文）。

### 5.4 校验规则

| 规则 | 位置 |
|---|---|
| `name` / `title` / `content` trim 后非空 | Rust 端，写库前 |
| `category_id` 必须存在 | DB 外键 + `update_category` 时显式 SELECT |
| `favorite` 默认 0 | Rust 端 default |

---

## 6. 前端架构

### 6.1 应用启动

```
main.ts
  ├── createApp(App)
  ├── 挂载到 #app
App.vue (onMounted)
  ├── await Promise.all([
  │     useCategories().load(),
  │     useSnippets().load()
  │   ])
  └── 注册键盘监听 useShortcuts.bindGlobal()
```

### 6.2 状态（spec §9.2）

```ts
// composables/useCategories.ts
const categories = ref<Category[]>([])
const selectedCategoryId = ref<number | null>(null)  // null = 全部 = 全局检索

// composables/useSnippets.ts
const snippets = ref<Snippet[]>([])
const selectedSnippetId = ref<number | null>(null)
const searchQuery = ref('')
const isEditing = ref(false)
```

> 所有 composables 模块级 `ref` 直接作为单例，不用 `provide/inject`，更不用 Pinia。
> 上限 5000 片段全量加载到内存即可，搜索过滤走 `computed`，无性能问题。

### 6.3 搜索（spec §9.4）

完全按 spec §9.4 的伪代码实现 `filteredSnippets` 这个 `computed`：

1. 若 `selectedCategoryId !== null` 先按 category_id 过滤；
2. 关键字 `keyword.toLowerCase()` 在 3 个字段中包含匹配（title/content/category_name）；
3. 排序：`favorite desc → last_used_at desc → used_count desc → updated_at desc`。

`category_name` 通过 `joinSnippetWithCategoryName(snippets, categories)` 在 composable 中合成。

### 6.4 复制流程

```
用户按 Enter →
  await navigator.clipboard.writeText(snippet.content)
  await invoke('mark_snippet_used', { id: snippet.id })
  本地更新 snippet.used_count、last_used_at（避免再调 list_snippets）
  toast: "已复制"
```

### 6.5 组件交互骨架

```
App.vue
├── SearchBar    （绑定 searchQuery）
├── CategoryList （v-for categories；点击选中 selectedCategoryId）
├── SnippetList  （v-for filteredSnippets；点击选中 selectedSnippetId）
└── SnippetDetail / SnippetEditor （根据 isEditing 切换）
```

---

## 7. 键盘交互（spec §11）

| 快捷键 | 作用域 | 实现 |
|---|---|---|
| 可配置呼出键 | 全局 | `tauri-plugin-global-shortcut`，键位由用户在设置里录制（默认未注册）；调用 `toggle_main_window` |
| `Esc` | 窗口 | `useShortcuts` 监听；优先关闭编辑器 → 否则隐藏窗口 |
| `Enter` | 窗口 | 复制选中片段（见 §6.4） |
| `Ctrl+N` | 窗口 | 打开新增片段表单 |
| `Ctrl+E` | 窗口 | 编辑当前选中片段 |
| `Ctrl+D` | 窗口 | 删除当前选中片段（带确认） |
| `Ctrl+Shift+N` | 窗口 | 新增分类（弹出输入框） |
| `↑/↓` | 窗口 | 切换 `selectedSnippetId`（在 `filteredSnippets` 上） |
| `Ctrl+↑/Ctrl+↓` | 窗口 | 切换 `selectedCategoryId`（在 `categories` 上 + `null`/全部） |

约定：编辑器打开时，除 `Esc` 外其他快捷键暂停（典型的 modal 行为）。

---

## 8. UI 设计

### 8.1 布局（呼应 spec §10.1 + UI 草图）

```
┌──────────────────────────────────────────────────────────────────┐
│ Search snippets, commands...                                      │
├──────────────┬─────────────────────────┬─────────────────────────┤
│              │                         │                         │
│ 全部         │  查看 GPU 温度          │  标题：查看 GPU 温度     │
│ 默认         │  kubectl 查看 Pod       │  分类：ROCm              │
│ Linux        │  docker logs            │  收藏                  │
│ Docker       │  md5 校验文件           │                         │
│ Kubernetes   │                         │  rocm-smi --showtemp     │
│ ROCm  ▸      │                         │                         │
│              │                         │                         │
├──────────────┴─────────────────────────┴─────────────────────────┤
│ Enter 复制 │ Esc 关闭 │ Ctrl+N 新增 │ Ctrl+E 编辑                │
└──────────────────────────────────────────────────────────────────┘
```

宽度比例（建议）：分类栏 18% / 列表 32% / 详情 50%；最小窗口 800×500。

### 8.2 视觉令牌

```css
:root {
  --bg-1: #0f1115;        /* 整体背景 */
  --bg-2: #161a21;        /* 面板 */
  --bg-3: #1d222b;        /* 选中态 */
  --fg-1: #e5e9f0;
  --fg-2: #9aa3b2;
  --accent: #5b8def;
  --border: #242a35;
  --mono: 'JetBrains Mono', 'Cascadia Code', Consolas, monospace;
}
```

理由：v1 选深色主题（命令行使用者偏好），亮色主题留作 v2。

### 8.3 内容预览

详情区的 `content` 用等宽字体 + `<pre>` 渲染，保留换行；不做语法高亮（v2 引入 highlight.js 或 shiki 时再加）。

---

## 9. 关键流程时序

### 9.1 创建片段

```
SnippetEditor.vue (用户填表 → 点击保存)
  → useSnippets().create(payload)
    → invoke('create_snippet', payload)
      → Rust：校验 → INSERT → SELECT 回查 → 返回 Snippet
    ← 返回 Snippet
  → snippets.value.push(newSnippet)
  → selectedSnippetId.value = newSnippet.id
  → isEditing.value = false
```

### 9.2 全局检索

```
SearchBar 输入 "pod"
  → searchQuery.value = "pod"
  → filteredSnippets (computed) 自动重算
  → SnippetList 重新渲染
```

无 invoke 调用，纯前端 reactivity。

### 9.3 删除分类（含校验）

```
CategoryList.vue (右键删除)
  → 确认对话框
  → useCategories().delete(id)
    → invoke('delete_category', { id })
      → Rust：SELECT count FROM snippets WHERE category_id=? 
              若 > 0 → Err(CategoryNotEmpty)
              否则 DELETE FROM categories WHERE id=?
    ← 成功 / Err
  → 成功：从 categories.value 中移除；如果 selectedCategoryId 等于该 id，重置为 null
  → 失败：toast "分类不为空，请先删除其下片段"
```

---

## 10. 验收对照（呼应 spec §13）

实现完成后，逐条勾选 spec §13 的验收清单。本文档不再复述。**额外**新增一组工程级验收：

- [ ] 关闭并重启应用，所有数据保持
- [ ] 数据库被外部删除后启动，能自动重建并落入"默认"分类
- [ ] 5000 条片段下，搜索关键字键入响应 < 50ms
- [ ] 呼出快捷键在窗口最小化/失焦时也能显隐（用户在设置里录制）
- [ ] 中文文案、中文分类名均能搜索命中
- [ ] `delete_category` 对非空分类返回的错误文案在 UI 上展示

---

## 11. 风险与未决事项

| 项 | 风险 | 处置 |
|---|---|---|
| `search_snippets` / `copy_snippet` 未实现 | 与 spec §7 命令清单形式上不齐 | §5.2 已标注待用户确认 |
| 前端内存搜索 | 数据量超阈值后退化 | v1 假设 < 5000；v2 切换到 SQL/FTS5，仅改 `useSnippets.search()` 的实现 |
| 全局快捷键冲突 | 不同系统可能与 IME 冲突 | v1 由用户在设置里录制（注册失败时提示），避免默认键抢占 |
| 数据备份/导出 | 用户数据无导出渠道 | v1 文档化数据库文件位置；v2 加 export/import |

---

## 12. 实现里程碑（建议）

1. **M1 框架**：Tauri 工程、表结构、`Mutex<Connection>` State、empty App.vue 启动可见
2. **M2 分类 CRUD**：左栏功能完成，能新增/重命名/删除空分类
3. **M3 片段 CRUD**：编辑器、列表、详情；可创建/编辑/删除片段
4. **M4 搜索 + 排序**：spec §9.4 全部实现
5. **M5 键盘 + 复制**：spec §11 + §6.4 复制流
6. **M6 收尾**：UI 细节、空态、验收清单

每个里程碑都应该可运行、可点击、可让用户验证。

---

## 13. 与 target.md 的偏差汇总

实现前请用户拍板的 3 处偏差，集中在此（其余决策都严格遵循 target.md）：

1. **不实现 `search_snippets` 后端命令**（spec §7.2 列出）—— 改为前端 `computed`。
2. **不实现 `copy_snippet` 后端命令**（spec §7.2 列出）—— 改为前端剪贴板 + `mark_snippet_used`。
3. **呼出快捷键由用户录制而非固定 `Ctrl+Space`** —— spec §11 原写固定 Ctrl+Space，现已改为可配置（用户在设置弹窗录制），因为默认键易与 IME 冲突。

如果其中任何一项不被接受，会在 §5.2 / §6.4 / §7 中按要求改回。
