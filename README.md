# vPlayer

基于 `Tauri 2 + Vue 3 + Rust` 的桌面视频播放器项目。

## 首次启动（最快 3 条命令）

```bash
cd frontend
pnpm install
pnpm tauri:dev
```

如果启动成功，你会看到桌面窗口，并且前端开发服务运行在 `http://localhost:5175`。

## 环境要求

- pnpm tauri devNode.js 18+（建议 20+）
- `pnpm`
- Rust 工具链（`rustup`, `cargo`）
- 平台依赖（Tauri 运行依赖，按官方文档安装）

## 项目结构

- `frontend/`：Vue + Vite 前端
- `src-tauri/`：Tauri + Rust 后端
- `docs/`：架构与执行文档

## 本地启动（开发模式）

1. 安装前端依赖

```bash
cd frontend
pnpm install
```

1. 启动桌面应用（会自动拉起 Vite dev server + Tauri）

```bash
pnpm tauri:dev
```

## 单独启动前端（可选）

如果你只想调 UI，不启动桌面壳：

```bash
cd frontend
pnpm dev
```

默认地址：`http://localhost:5175`

## 构建

1. 构建前端资源

```bash
cd frontend
pnpm build
```

1. 构建桌面应用

```bash
cd src-tauri
cargo build
```

## 当前状态

当前处于基础架构搭建阶段，核心播放链路仍在逐步迁移（命令已可调用，`libmpv` 深度集成仍在进行中）。

更多细节见：

- `docs/architecture.md`
- `docs/implementation-alignment-checklist.md`
- `docs/week1-pr-plan.md`

## FAQ（常见问题）

### 1) `cargo check` 报 icon 错误

现象示例：

- `failed to open icon .../src-tauri/icons/32x32.png`
- `icon ... is not RGBA`

处理：

- 确认文件存在：`src-tauri/icons/32x32.png`
- 确认 `src-tauri/tauri.conf.json` 中 `bundle.icon` 路径正确
- 如果你替换了图标，确保是有效 RGBA PNG

### 2) `pnpm tauri dev` 启动失败（依赖或命令缺失）

先确认依赖安装完成：

```bash
cd frontend
pnpm install
```

检查 Tauri CLI 是否可用（项目内）：

```bash
cd frontend
pnpm tauri -v
```

若仍失败，通常是系统层 Tauri 运行依赖缺失，请按 Tauri 官方文档补齐平台依赖后重试。

### 3) `5175` 端口被占用

现象：前端 dev server 无法启动，提示端口冲突。

处理方式 A（推荐）：结束占用进程后重试。

处理方式 B：改端口（例如 `5174`），同时同步 `src-tauri/tauri.conf.json` 的 `build.devUrl`。

### 4) 只改前端样式，是否必须启动 Tauri？

不必须。你可以只跑前端：

```bash
cd frontend
pnpm dev
```

等需要联调 Tauri command 时，再使用：

```bash
cd frontend
pnpm tauri:dev
```
