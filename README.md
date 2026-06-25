# SnipDock

SnipDock 是一个本地优先的桌面记录工具，用来保存、检索和快速复制任何常用内容：命令、文本、链接、路径、模板、备注、提示词都可以放进去。

它适合经常在终端、IDE、浏览器、工单、文档之间切换的人：把零散信息集中保存到本地，通过分类、搜索和快捷键快速找回。

> 当前版本：`v0.1.0`  
> 平台：Windows x64（Tauri 2 + Vue 3 + SQLite）

---

## 功能亮点

### 本地优先

所有数据保存在本机 SQLite 数据库中，不需要登录账号，也不会上传到云端。

### 分类组织

用分类管理记录，例如：

- Linux
- Docker
- Kubernetes
- Git
- SQL
- 排障命令
- 常用路径
- 文本模板

每条记录都属于一个分类。第一版保持简单，不做多级分类。

### 快速搜索与高亮

主窗口支持按以下字段搜索：

- 标题
- 内容
- 分类名称

搜索结果会高亮命中的字符，方便快速确认目标记录。

### 快速呼出窗口

可以在设置中录制一个全局快捷键，随时呼出轻量搜索窗口。

快速窗口采用两栏布局：

- 左侧：分类 / 收藏 / 全部
- 右侧：当前分类下的记录

默认进入「收藏」视图，并按使用频率排序，适合像 Alfred / Raycast 一样快速复制常用内容。

### 一键复制与使用频率排序

选中记录后按 `Enter` 即可复制到剪贴板。复制后会记录使用次数，常用记录会排在更靠前的位置。

### 收藏

可以把高频记录加入收藏，在主窗口和快速窗口中优先访问。

### 系统托盘常驻

关闭主窗口不会退出应用，而是隐藏到系统托盘。需要时可以通过托盘或全局快捷键再次唤起。

### 数据导出 / 导入

设置面板内提供 JSON 导出 / 导入能力：

- 导出：把全部分类与记录保存为一个 JSON 文件
- 导入：合并到当前数据库，分类按名称复用，记录自动跳过重复项

适合备份、迁移到新机器，或者在多台电脑之间手动同步。

---

## 安装

### 从 Release 下载

前往 GitHub Releases 下载最新安装包：

```text
SnipDock_0.1.0_x64-setup.exe
```

也可以下载 MSI 版本：

```text
SnipDock_0.1.0_x64_en-US.msi
```

普通用户建议使用 `setup.exe`。

---

## 快速开始

1. 启动 SnipDock
2. 新建分类，例如「链接」「提示词」「排障命令」或「文本模板」
3. 新增记录，填写标题与内容
4. 在顶部搜索框输入关键字
5. 选中记录，按 `Enter` 复制
6. 在设置中录制全局快捷键，之后可随时呼出快速窗口

---

## 快捷键

| 快捷键 | 功能 |
|---|---|
| `Enter` | 复制当前选中记录 |
| `Esc` | 关闭编辑器 / 隐藏窗口 |
| `Ctrl + N` | 新增记录 |
| `Ctrl + E` | 编辑当前记录 |
| `Ctrl + D` | 删除当前记录 |
| `↑ / ↓` | 切换记录 |
| `Ctrl + ↑ / Ctrl + ↓` | 切换分类 |
| 自定义全局快捷键 | 呼出 / 隐藏快速搜索窗口 |

全局快捷键在「设置」里录制，避免和系统或输入法快捷键冲突。

---

## 数据与隐私

SnipDock 的数据默认保存在系统应用数据目录下：

| 系统 | 数据库路径 |
|---|---|
| Windows | `%APPDATA%\com.snipdock.app\snipdock.db` |
| macOS | `~/Library/Application Support/com.snipdock.app/snipdock.db` |
| Linux | `~/.local/share/com.snipdock.app/snipdock.db` |

删除该文件相当于清空所有数据；下次启动会自动重建数据库并创建「默认」分类。

如果要迁移数据，优先使用设置面板里的「导出 / 导入」功能，而不是直接复制数据库文件。

---

## 开发

### 环境要求

- Node.js ≥ 18
- Rust 最新稳定版 + Cargo
- Tauri 2 平台依赖：[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### 启动开发环境

```bash
npm install
npm run tauri dev
```

### 构建发布包

```bash
npm run tauri build
```

Windows 构建产物通常位于：

```text
src-tauri/target/release/bundle/nsis/
src-tauri/target/release/bundle/msi/
```

---

## 技术栈

- Tauri 2
- Vue 3
- TypeScript
- Rust
- SQLite / rusqlite
- @element-plus/icons-vue

---

## 文档

- 产品规格：[docs/target.md](docs/target.md)
- 第一版设计文档：[docs/design.md](docs/design.md)
- 开发工作指引：[CLAUDE.md](CLAUDE.md)

---

## 当前版本范围

`v0.1.0` 聚焦本地记录管理的核心体验：

- 分类
- 记录 CRUD
- 搜索与高亮
- 收藏
- 快速呼出窗口
- 系统托盘
- 可配置全局快捷键
- JSON 导出 / 导入

第一版刻意保持轻量，不做多级分类、全文索引、云同步或团队协作。
