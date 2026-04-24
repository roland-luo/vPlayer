# Week 1 PR 切分计划（As-Is -> Target）

> 目标：在不破坏现有可运行状态的前提下，完成 Week 1 的结构对齐与 IPC 迁移。  
> 原则：小步提交、每个 PR 可独立回滚、每个 PR 都有可验证结果。

---

## PR-1：后端模块骨架落位

**目标**
- 把 Target 目录骨架创建出来，但不引入业务行为变化。

**改动范围**
- 新增（空实现/占位）：
  - `src-tauri/src/mpv/{mod.rs,core.rs,renderer.rs,event.rs,options.rs}`
  - `src-tauri/src/render/{mod.rs,gl_context.rs,texture.rs,frame.rs}`
  - `src-tauri/src/ipc/{mod.rs,commands.rs,events.rs}`
  - `src-tauri/src/error/{mod.rs,fallback.rs,user_notify.rs}`
  - `src-tauri/src/utils/{mod.rs,logging.rs,paths.rs}`
- 调整：
  - `src-tauri/src/main.rs` 仅增加 `mod ...` 声明（若需要）

**验收标准**
- `cargo check` 通过。
- 应用可正常启动（即便功能仍是 stub）。

**风险**
- 模块声明路径错误导致编译失败。

**回滚点**
- 回滚新增目录与 `main.rs` 的模块声明。

**建议提交信息**
- `refactor(backend): scaffold target module layout`

---

## PR-2：IPC 命令迁移到 `ipc/commands.rs`

**目标**
- 把 command 实现从 `main.rs` 挪到 `ipc/commands.rs`，行为不变。

**改动范围**
- 修改：
  - `src-tauri/src/main.rs`
  - `src-tauri/src/ipc/commands.rs`
  - `src-tauri/src/ipc/mod.rs`
- 保持不变：
  - `frontend/src/api/player.ts`（命令名不改）

**迁移命令**
- `play_file`
- `pause`
- `resume`
- `seek`
- `set_volume`
- `get_player_state`

**验收标准**
- 前端调用以上命令全部成功。
- `main.rs` 不再包含命令函数实现，仅做注册与启动装配。

**风险**
- `tauri::generate_handler![]` 引用路径错误。

**回滚点**
- 将命令函数搬回 `main.rs`，恢复旧注册。

**建议提交信息**
- `refactor(ipc): move tauri commands out of main`

---

## PR-3：统一后端状态模型（最小版）

**目标**
- 为 `get_player_state` 建立单一返回模型，固定字段语义。

**改动范围**
- 新增/修改：
  - `src-tauri/src/ipc/commands.rs`（或新增 `state` 模块）
  - `src-tauri/src/main.rs`（仅装配变化）
  - `frontend/src/api/player.ts`（仅类型注释同步，行为不变）

**最小字段**
- `state`
- `position`
- `duration`
- `volume`（固定 `0~100`）

**验收标准**
- `get_player_state` 返回结构稳定。
- 前端显示与后端口径一致。

**风险**
- 未来从 `0~100` 迁移到 `0~1` 时兼容性问题。

**回滚点**
- 恢复原始 JSON 拼接实现。

**建议提交信息**
- `refactor(state): normalize player state payload`

---

## PR-4：事件通道占位（不切行为）

**目标**
- 建立 `ipc/events.rs` 的最小接口，为 Week 2 接事件流做准备。

**改动范围**
- 新增/修改：
  - `src-tauri/src/ipc/events.rs`
  - `src-tauri/src/ipc/mod.rs`
- 允许先提供占位函数：
  - `emit_player_state_change(...)`
  - `emit_player_progress(...)`
  - `emit_video_error(...)`

**验收标准**
- 编译通过。
- 无行为回归（当前 UI 仍可工作）。

**风险**
- 提前引入未使用代码警告（可通过 `#[allow(dead_code)]` 暂时处理）。

**回滚点**
- 删除事件占位接口，回到纯命令模型。

**建议提交信息**
- `chore(ipc): add event emission stubs for week2`

---

## PR-5：文档回写（同步 As-Is）

**目标**
- 将 Week 1 已完成项回写到文档，防止再次漂移。

**改动范围**
- 修改：
  - `docs/architecture.md`
  - `docs/implementation-alignment-checklist.md`

**验收标准**
- 文档中的 As-Is 与代码一致。
- 所有“已完成”条目有可对应代码位置。

**风险**
- 文档更新滞后于代码。

**回滚点**
- 仅文档回滚，不影响功能代码。

**建议提交信息**
- `docs(architecture): sync as-is after week1 refactor`

---

## 合并顺序（必须按序）

1. PR-1 模块骨架  
2. PR-2 IPC 迁移  
3. PR-3 状态统一  
4. PR-4 事件占位  
5. PR-5 文档回写

---

## Week 1 退出条件（Exit Criteria）

- [x] `main.rs` 无业务命令实现，仅负责装配。
- [x] 命令逻辑集中在 `ipc/commands.rs`。
- [x] Target 模块目录存在且可编译。
- [x] `get_player_state` 字段口径固定（`volume`=0~100）。
- [x] 文档完成回写并与代码一致。
