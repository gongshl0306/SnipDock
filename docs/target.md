# SnipDock 分类与检索功能 Spec 增补

## 1. 功能目标

SnipDock 需要支持用户自定义分类，每个分类下可以保存多个片段记录。每条片段包含标题、内容、所属分类、是否收藏等字段。

用户可以在两种范围内检索：

```text
全局检索：搜索所有分类下的片段
分类内检索：只搜索当前分类下的片段
```

检索字段包括：

```text
片段标题
片段内容
分类名称
```

---

## 2. 分类设计

### 2.1 分类概念

分类用于组织片段，类似文件夹。

示例分类：

```text
Linux
Docker
Kubernetes
Git
SQL
ROCm
排障命令
常用路径
文本模板
```

### 2.2 分类规则

1. 用户可以创建分类。
2. 用户可以重命名分类。
3. 用户可以删除分类。
4. 每条片段必须属于一个分类。
5. 分类名称不可为空。
6. 同级分类名称第一版可以不强制唯一，但建议唯一。
7. 删除分类时，需要处理分类下已有片段。

### 2.3 删除分类策略

第一版建议采用安全策略：

```text
如果分类下存在片段，不允许直接删除。
提示用户先移动或删除该分类下的片段。
```

后续可以增加：

```text
删除分类并删除所有片段
删除分类并移动片段到默认分类
```

---

## 3. 默认分类

首次启动时创建默认分类：

```text
默认
```

也可以内置一些示例分类：

```text
命令
代码
SQL
路径
模板
```

建议第一版只创建：

```text
默认
```

避免过度设计。

---

## 4. 组织方式

片段只通过**分类**组织。v1 不引入 Tag、标签或多维度归类。

如果未来确实需要跨分类聚合，会在 v2 重新评估，不在本文件预留接口。

---

## 5. 数据库设计

### 5.1 categories 表

新增分类表：

```sql
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### 5.2 snippets 表

snippets 表增加 `category_id` 字段：

```sql
CREATE TABLE IF NOT EXISTS snippets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    favorite INTEGER DEFAULT 0,
    used_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_used_at TEXT,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);
```

### 5.3 默认分类初始化

应用首次启动时，如果 categories 表为空，自动创建默认分类：

```sql
INSERT INTO categories (
    name,
    description,
    sort_order,
    created_at,
    updated_at
) VALUES (
    '默认',
    '默认分类',
    0,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);
```

### 5.4 索引设计

为了提高查询速度，增加索引：

```sql
CREATE INDEX IF NOT EXISTS idx_snippets_category_id
ON snippets(category_id);

CREATE INDEX IF NOT EXISTS idx_snippets_updated_at
ON snippets(updated_at);

CREATE INDEX IF NOT EXISTS idx_snippets_used_count
ON snippets(used_count);

CREATE INDEX IF NOT EXISTS idx_snippets_last_used_at
ON snippets(last_used_at);
```

---

## 6. TypeScript 类型设计

### 6.1 Category 类型

```ts
export interface Category {
  id: number
  name: string
  description: string
  sort_order: number
  created_at: string
  updated_at: string
}
```

### 6.2 Snippet 类型

```ts
export interface Snippet {
  id: number
  category_id: number
  category_name?: string
  title: string
  content: string
  favorite: number
  used_count: number
  created_at: string
  updated_at: string
  last_used_at?: string
}
```

### 6.3 CreateSnippetPayload

```ts
export interface CreateSnippetPayload {
  category_id: number
  title: string
  content: string
  favorite?: number
}
```

### 6.4 UpdateSnippetPayload

```ts
export interface UpdateSnippetPayload {
  id: number
  category_id: number
  title: string
  content: string
  favorite?: number
}
```

---

## 7. Rust Command 接口设计

### 7.1 分类相关 Commands

```text
list_categories
create_category
update_category
delete_category
```

### 7.2 片段相关 Commands

```text
list_snippets
list_snippets_by_category
search_snippets
create_snippet
update_snippet
delete_snippet
copy_snippet
mark_snippet_used
```

### 7.3 推荐接口定义

#### list_categories

返回所有分类，按 `sort_order` 和 `created_at` 排序。

#### create_category

输入：

```json
{
  "name": "Kubernetes",
  "description": "K8s 常用命令"
}
```

#### update_category

输入：

```json
{
  "id": 2,
  "name": "K8s",
  "description": "Kubernetes 常用命令"
}
```

#### delete_category

输入：

```json
{
  "id": 2
}
```

如果分类下存在片段，返回错误：

```text
Category is not empty
```

#### search_snippets

输入：

```json
{
  "query": "gpu temp",
  "categoryId": null
}
```

说明：

```text
categoryId = null 表示全局检索
categoryId = 具体数字 表示分类内检索
```

---

## 8. 搜索设计

### 8.1 搜索范围

搜索字段包括：

```text
title
content
category_name
```

### 8.2 全局检索

全局检索时，搜索所有分类下的片段。

伪逻辑：

```text
从全部 snippets 中搜索
匹配 title / content / category_name
返回结果中展示分类名
```

适用场景：

```text
用户不知道片段在哪个分类
用户想快速搜索所有命令
```

### 8.3 分类内检索

分类内检索时，只搜索当前分类下的片段。

伪逻辑：

```text
先过滤 category_id
再匹配 title / content
返回当前分类内结果
```

适用场景：

```text
用户已经知道要找 Kubernetes 命令
用户只想在 Docker 分类下搜索 logs
```

### 8.4 高亮命中字符

搜索框非空时，列表与详情中命中的子串使用 `<mark>` 包裹高亮显示。
匹配规则与过滤一致：大小写不敏感的子串匹配；空白分隔的多关键字逐个高亮。

---

## 9. 前端搜索策略

### 9.1 第一版策略

第一版仍然保持轻量：

```text
启动时加载 categories
启动时加载 snippets
前端内存中完成搜索和过滤
新增 / 编辑 / 删除时同步 SQLite
```

### 9.2 前端状态设计

```ts
const categories = ref<Category[]>([])
const snippets = ref<Snippet[]>([])
const selectedCategoryId = ref<number | null>(null)
const searchQuery = ref('')
```

### 9.3 全局 / 分类内检索状态

```ts
selectedCategoryId = null
```

表示全局检索。

```ts
selectedCategoryId = 3
```

表示只在分类 ID 为 3 的分类中检索。

### 9.4 搜索伪代码

```ts
const filteredSnippets = computed(() => {
  const keyword = searchQuery.value.trim().toLowerCase()
  const currentCategoryId = selectedCategoryId.value

  return snippets.value
    .filter(item => {
      if (currentCategoryId !== null && item.category_id !== currentCategoryId) {
        return false
      }

      if (!keyword) {
        return true
      }

      const categoryName = item.category_name || ''

      return (
        item.title.toLowerCase().includes(keyword) ||
        item.content.toLowerCase().includes(keyword) ||
        categoryName.toLowerCase().includes(keyword)
      )
    })
    .sort((a, b) => {
      if (b.favorite !== a.favorite) return b.favorite - a.favorite
      if ((b.last_used_at || '') !== (a.last_used_at || '')) {
        return (b.last_used_at || '').localeCompare(a.last_used_at || '')
      }
      if (b.used_count !== a.used_count) return b.used_count - a.used_count
      return b.updated_at.localeCompare(a.updated_at)
    })
})
```

---

## 10. UI 调整

### 10.1 主界面布局

增加左侧分类栏。

```text
+------------------------------------------------------------+
| Search snippets, commands...                                |
+----------------+-------------------------+-----------------+
| 分类            | 片段列表                 | 片段详情         |
| 全部            | 查看 GPU 温度             | 标题             |
| 默认            | kubectl 查看 Pod          | 分类             |
| Linux           | docker logs               | 收藏             |
| Docker          | md5 校验文件              | 内容预览         |
| Kubernetes      |                         |                 |
+----------------+-------------------------+-----------------+
| Enter 复制 | Esc 关闭 | Ctrl+N 新增 | Ctrl+E 编辑       |
+------------------------------------------------------------+
```

### 10.2 分类栏

分类栏展示：

```text
全部
默认
用户自定义分类 1
用户自定义分类 2
...
```

其中：

```text
全部 = 全局检索
具体分类 = 分类内检索
```

### 10.3 分类操作

分类栏支持：

```text
新增分类
重命名分类
删除分类
```

可以通过右键菜单或顶部按钮实现。

### 10.4 片段编辑器

新增 / 编辑片段时，需要选择分类。

字段：

```text
分类
标题
内容
是否收藏
```

---

## 11. 交互快捷键调整

新增分类相关快捷键：

```text
Ctrl + Space：呼出 / 隐藏窗口
Esc：隐藏窗口
Enter：复制当前选中片段
Ctrl + N：新增片段
Ctrl + E：编辑当前片段
Ctrl + D：删除当前片段
Ctrl + Shift + N：新增分类
↑ / ↓：切换片段
Ctrl + ↑ / Ctrl + ↓：切换分类
```

---

## 12. 典型用户流程

### 12.1 创建分类

```text
用户打开 SnipDock
点击新增分类
输入分类名：Kubernetes
保存
左侧分类栏出现 Kubernetes
```

### 12.2 创建片段

```text
用户点击新增片段
选择分类：Kubernetes
标题：查看所有 Pod
内容：kubectl get pods -A -o wide
保存
```

### 12.3 全局检索

```text
用户选择：全部
搜索：pod
结果显示所有分类中 title/content 包含 pod 的片段
```

### 12.4 分类内检索

```text
用户选择：Kubernetes
搜索：pod
结果只显示 Kubernetes 分类下匹配 pod 的片段
```

### 12.5 命中高亮

```text
用户搜索：pod
列表与详情中所有 pod 子串被高亮（不区分大小写）
```

---

## 13. 验收标准增补

### 13.1 分类验收

```text
可以创建分类
可以重命名分类
可以删除空分类
不能误删包含片段的分类
可以在片段编辑器中选择分类
片段保存后归属于正确分类
```

### 13.2 搜索高亮验收

```text
搜索关键字命中的子串被 <mark> 标签高亮显示
高亮匹配大小写不敏感
空白分隔的多关键字逐个高亮
搜索框为空时不显示任何高亮
```

### 13.3 搜索验收

```text
可以全局搜索所有分类的片段
可以只在当前分类内搜索片段
可以根据 title 搜索
可以根据 content 搜索
全局搜索结果需要展示片段所属分类
```

---

## 14. 对原 Spec 的影响

需要更新以下模块：

```text
数据库设计：新增 categories 表，snippets 增加 category_id
UI 设计：增加左侧分类栏
前端状态：增加 selectedCategoryId
搜索逻辑：支持全局搜索和分类内搜索
Rust Commands：增加分类 CRUD
片段编辑器：增加分类选择
验收标准：增加分类检索验收
```

---

## 15. 第一版推荐实现方案

为了保持轻量，第一版采用以下实现：

```text
分类使用 categories 表
片段通过 category_id 关联分类
搜索在前端内存中完成
全局搜索和分类内搜索通过 selectedCategoryId 控制
```

不建议第一版就做：

```text
高级搜索语法
全文索引
多级分类
分类树
```

这些可以放到后续版本。

---

## 16. 数据导出 / 导入

### 16.1 目的

为了让用户能跨机器迁移数据、做手动备份，第一版在设置弹窗里提供
**导出** 与 **导入** 两个按钮。

### 16.2 文件格式

单文件 JSON，UTF-8 编码：

```json
{
  "version": 1,
  "exported_at": "<RFC3339>",
  "categories": [
    { "name": "Linux", "description": "" }
  ],
  "snippets": [
    {
      "category_name": "Linux",
      "title": "查看 GPU 温度",
      "content": "rocm-smi --showtemp",
      "favorite": 0
    }
  ]
}
```

字段说明：

```text
version       固定为 1，用于将来格式演进
exported_at   导出时的 RFC3339 时间戳
categories    [{ name, description }] —— 不导出 id / sort_order / 时间戳
snippets      [{ category_name, title, content, favorite }] —— 用 name 关联分类
```

不导出：`id`、`sort_order`、`created_at`、`updated_at`、`last_used_at`、`used_count`。
这些都是本地状态，跨机器没意义；导入时由当前库重新生成。

### 16.3 导入策略

合并语义，**不清空当前数据**：

```text
分类按 name 唯一：
  已存在  → 复用原 id
  不存在  → 新建（沿用 backup 中的 description）

片段按 (匹配后 category_id, title, content) 三元组去重：
  已存在  → 跳过
  不存在  → 插入

backup 中的分类名在本地都找不到（罕见）→ 落到「默认」分类
```

整个导入过程包在 SQLite 事务里，任一步失败整体回滚。

### 16.4 验收

```text
导出后生成的 JSON 能用任意文本编辑器打开并阅读
反复导入同一个文件不会产生重复片段
导出后清空 DB，再导入，数据与之前一致（不要求 id / 时间戳一致）
导入过程中 SQL 失败 → 数据库回滚到导入前状态
```
