# SnipDock

一个本地的代码片段管理器。键盘驱动、按分类组织、可选 Tag、模糊搜索、一键复制。

## 文档

- 产品规格：[docs/target.md](docs/target.md)
- 第一版设计文档：[docs/design.md](docs/design.md)
- 给 Claude Code 的工作指引：[CLAUDE.md](CLAUDE.md)

## 开发环境

- Node.js ≥ 18
- Rust（最新稳定版）+ Cargo
- 平台依赖：参考 [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

## 启动

```bash
npm install
npm run tauri dev
```

## 构建

```bash
npm run tauri build
```

## 数据库位置

应用首次启动会在系统的 app data 目录下创建 `snipdock.db`：

- Windows：`%APPDATA%\com.snipdock.app\snipdock.db`
- macOS：`~/Library/Application Support/com.snipdock.app/snipdock.db`
- Linux：`~/.local/share/com.snipdock.app/snipdock.db`

删除该文件相当于清空所有数据；下次启动会自动重建并落入"默认"分类。
