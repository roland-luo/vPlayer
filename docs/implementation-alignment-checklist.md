# vPlayer 对齐实现清单

> 目标：把当前 `As-Is`（Week 1 已完成结构对齐）逐步对齐到 `architecture.md` 的 `Target` 架构。  
> 适用范围：`src-tauri/` 与 `frontend/src/`。  
> 约束：本清单默认保持当前 IPC 命令名（`play_file` / `get_player_state` 等）不变，避免无效重命名成本。

---

## 0. 对齐原则

- 先做“结构落位”，再做“功能扩展”。
- 每周只引入一个主要风险面（渲染、事件、插件、错误恢复）。
- 每一项都必须有可验证验收标准，不接受“代码看起来差不多”。

---

## 1. Week 1：结构对齐（P0）

### 1.1 后端模块骨架落位

- [x] 新建目录与模块文件：
  - `src-tauri/src/mpv/{mod.rs,core.rs,renderer.rs,event.rs,options.rs}`
  - `src-tauri/src/render/{mod.rs,gl_context.rs,texture.rs,frame.rs}`
  - `src-tauri/src/ipc/{mod.rs,commands.rs,events.rs}`
  - `src-tauri/src/error/{mod.rs,fallback.rs,user_notify.rs}`
  - `src-tauri/src/utils/{mod.rs,logging.rs,paths.rs}`
- 验收标准：
  - `cargo check` 通过。
  - `main.rs` 不再承载具体业务逻辑，仅保留组装与启动。

### 1.2 IPC 入口迁移

- [x] 将当前命令实现从 `main.rs` 迁移到 `ipc/commands.rs`：
  - `play_file`
  - `pause`
  - `resume`
  - `seek`
  - `set_volume`
  - `get_player_state`
- 验收标准：
  - 前端 `frontend/src/api/player.ts` 无需改动即可调用成功。
  - 命令注册统一通过 `ipc::commands` 导出。

### 1.3 状态模型统一

- [x] 在后端定义 `PlayerState`（最小字段集）并集中管理：
  - `state`
  - `position`
  - `duration`
  - `volume`
- [x] 明确 `volume` 口径为 `0~100`（现状口径），并在接口注释中固定。
- 验收标准：
  - `get_player_state` 返回结构稳定。
  - 前端展示值与后端值一致，不出现比例换算分歧。

---

## 2. Week 2：事件流对齐（P1）

### 2.1 后端事件推送管道

- [x] 在 `ipc/events.rs` 增加基础事件发布（占位接口 + 命令路径联调已完成）：
  - `player:state_change`
  - `player:progress`
  - `video:error`（可先空实现占位）
- 验收标准：
  - 前端可订阅并收到事件。
  - 播放状态切换时事件能正确触发。

### 2.2 前端状态从“本地模拟”迁移为“后端事件主导”

- [x] 在 `frontend/src/App.vue` 接入后端事件监听。
- [x] 将 `isPlaying/currentTime/duration` 的更新源改为后端事件（保留短期 UI 补偿）。
- 验收标准：
  - 刷新后首次状态由 `get_player_state` 初始化。
  - 播放/暂停/seek 后 UI 与后端一致，不出现双源漂移。

---

## 3. Week 3：错误与回退对齐（P1）

### 3.1 启动失败回退

- [x] 在 `error/fallback.rs` 增加启动阶段错误封装与用户可见消息。
- [x] 出错时发出 `app:fatal_error`（最小载荷可先 `stage + message`）。
- 验收标准：
  - 人工注入初始化失败时，前端可见错误信息，不是静默失败。

### 3.2 渲染错误上报

- [x] 在渲染路径增加 `video:error` 事件出口。
- [x] 前端增加错误 overlay 占位组件（可先简版）。
- 验收标准：
  - 人工模拟渲染失败时，用户看到错误提示且应用不崩溃。

---

## 4. Week 4：插件边界最小落地（P2）

### 4.1 插件总线骨架

- [x] 新建 `plugin/bus.rs` 并实现最小接口：
  - `emit(event)`
  - `invoke(command)`
- [x] 为 `emit` 增加单插件错误隔离（先日志 + 不影响主流程）。
- 验收标准：
  - 插件 A 失败不影响插件 B。
  - 失败信息可在日志中定位。

### 4.2 插件注册占位

- [x] 新建 `plugin/registry.rs`，支持最小注册/查询。
- 验收标准：
  - `list_plugins` 能返回最小结构（可为空列表）。

---

## 5. 文档对齐任务（持续）

- [x] 每完成一个周阶段，回写 `docs/architecture.md` 的 As-Is 状态段。
- [x] 若新增/改名 IPC 命令，必须同步更新：
  - `docs/architecture.md`
  - `frontend/src/api/player.ts`
  - `src-tauri/src/ipc/commands.rs`
- [ ] 任何 `volume` 口径变更必须跨前后端与文档一次性提交。

---

## 6. 完成定义（Definition of Done）

以下条件全部满足，才算“完成对齐”：

- [ ] 代码结构与 `architecture.md` Target 目录一致（允许内部实现仍为最小版）。
- [ ] `main.rs` 仅做启动与依赖装配，不含业务命令实现。
- [ ] 前端状态主来源为后端事件，不再依赖本地模拟作为主逻辑。
- [ ] 启动失败与渲染失败存在用户可见错误反馈。
- [ ] 核心命令与字段口径在“代码-前端-文档”三处一致。

---

## 7. 推荐执行顺序（最小风险）

1. Week 1 先完成结构与 IPC 迁移。  
2. Week 2 再接事件流，解决状态漂移。  
3. Week 3 补错误回退，避免黑盒失败。  
4. Week 4 再引入插件边界，避免过早复杂化。
