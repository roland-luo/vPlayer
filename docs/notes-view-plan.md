# 笔记视图改造计划（Week 1 验证楔子）

> 日期：2026-05-12
> 上游依据：`docs/office-hours-design-20260422-164538.md`（Status: APPROVED）
> 状态：READY_FOR_ENG_REVIEW — design-review 4 项 BLOCKER 已全部解决（见末尾 §BLOCKER 解除状态）
> 范围：仅本周可验证闭环。不在本周做"多平台 / 富文本编辑器 / 跨视频笔记搜索"。
> 选定版式：Variant A（顶部输入 + 下方列表，改造现 BookmarkView，容器 PluginPopup 420×380）

---

## 背景

上一轮 office-hours 把"最小可验证楔子"定义为：

> 播放 + **时间戳笔记** + **一键回跳** + **笔记点击回放**

但近 3 周的 commit（`基础播放` → `视频信息+播放速度` → `均衡器` → `持久化存储`）全部是通用播放器功能，**没有一行代码触及笔记楔子**。

好消息：`frontend/src/components/BookmarkView.vue` 已经实现了"添加 / 列表 / 点击跳转 / 删除"4 个核心动作的 60%，**改造它比新建一个组件成本低**。

---

## 目标（本周交付）

让一位真实"网课重度学习者"能够在 30 分钟任务内：

1. 边看课边记 ≥5 条时间戳笔记；
2. 任意点击一条笔记 → 视频回到对应时刻并自动播放；
3. 结束时导出笔记为可读 markdown。

**Go 阈值（来自 office-hours doc）：** ≥3/5 用户愿意继续使用 + 任务完成率 ≥80%。

---

## 5 个改造点

### 改造 1：输入区从"书签名"扩展为"笔记内容"

**As-Is**
- `frontend/src/components/BookmarkView.vue:5-11` 是一个单行 `<input>`，`maxlength="60"`，placeholder `"书签名称..."`
- 当前是"标记位置"心智模型

**To-Be**
- 多行 `<textarea>`，建议 `maxlength` ≥ 500（覆盖一条完整学习笔记）
- placeholder 改为引导记笔记的语气，例如 `"记一笔（Enter 保存，Shift+Enter 换行）"`
- 默认 2 行高度，输入超过自动撑高至 4 行，再多则内部滚动
- 顶部显示一条"当前视频位置"chip，例如 `00:14:32`，按 mono 字体 + tabular-nums

**验收**
- 输入 ≥3 行内容仍能正常保存
- 列表展示时换行内容可读（不挤成一行）

---

### 改造 2：开始写笔记 → 视频自动暂停

**As-Is**
- 写书签时视频继续播放，用户必须自己先按暂停

**To-Be**
- `textarea` 首次 `focus` 触发 `pause` 命令
- 保存或失焦 + 输入框为空时自动 `resume`（保留用户原本的播放/暂停状态）
- 提供"不要自动暂停"的开关（设置面板）以应对录制类用户

**验收**
- 真实任务中用户不需要手动暂停一次就完成 5 条笔记

---

### 改造 3：点击笔记 → 跳到 t-3s 并自动播放

**As-Is**
- `frontend/src/components/BookmarkView.vue:111-113` 当前 `seekToBookmark` 跳到精确 `bm.position`，且不主动 play

**To-Be**
- 点击列表项时 `seek(Math.max(0, position - 3))`，确保回放有上下文
- 跳转后自动 `resume`（不管原状态）
- t-3s 偏移量应做成常量（默认 3s，未来允许设置面板调）

**验收**
- 真实任务中用户回放时**听到笔记前 2~3 秒的内容**，不会"跳进去发现已经讲完"

---

### 改造 4：笔记绑定具体视频

**当前未验证（API 层）**
- 需要确认 `BookmarkEntry`（`frontend/src/api/player.ts`）是否已含 `video_id` / `media_path` 字段
- 列表 API `listBookmarks()` 当前是全局列表还是按视频过滤未知

**To-Be**
- 笔记按"当前播放视频"过滤展示
- 切换视频时列表自动重载
- 后端持久化字段需含视频标识（路径 hash 或 ID）
- 切换视频不能丢失已有笔记

**验收**
- 打开视频 A 写 3 条 → 切到视频 B 列表为空 → 切回视频 A 仍能看到那 3 条

---

### 改造 5：导出 markdown

**As-Is**
- 无导出能力

**To-Be**
- 列表底部一个"导出 .md"按钮（或复制到剪贴板）
- 输出格式（最小版本）：
  ```markdown
  # {视频文件名}

  - **00:14:32** 第一条笔记内容
  - **00:21:08** 第二条笔记内容
  ```
- 时间戳保持 mono / tabular-nums 风格的对齐
- 优先做"复制到剪贴板"，文件保存可放到 P2

**验收**
- 用户能把当次笔记粘贴到 Notion / Obsidian / 飞书，时间戳清晰可读

---

## 不做（本周明确出清单）

- 富文本 / markdown 实时渲染编辑
- 跨视频笔记搜索 / 全局 tag
- 协作 / 同步 / 云端
- 笔记编辑（先只支持 add / delete，编辑放到下一轮验证后）
- 截图嵌入笔记
- AI 摘要

---

## 设计与工程评审顺序

1. **`/plan-design-review`**（本步）— 确认笔记输入区视觉、列表项展示、空状态、错误态
2. **`/plan-eng-review`** — 确认 API 字段 / 持久化路径 / 自动暂停的时序边界
3. 落地 → 5 用户验证

---

## 风险

| 风险 | 影响 | 当前对策 |
|------|------|----------|
| ~~`BookmarkEntry` 未含 video_id，后端要改 schema~~ ✅ 已确认 `video: String` 字段存在，无需迁移 | — | — |
| 自动暂停打断"被动听课"用户 | UX 倒退 | 提供开关，默认开 |
| t-3s 偏移对短笔记不友好（一条笔记只占 1 秒） | 回放体验差 | 后续可改成"跳到最近一个静默间隔" |
| markdown 导出格式与 Notion / Obsidian 不完全兼容 | 复制后需手动整理 | 先用最小通用语法 |
| **新发现：** 容器是 PluginPopup 420×380，不是 280px sidebar | Variant A 视觉尺寸需重新核对 | 见下文 §视觉规格，宽度按 420 计 |
| **新发现：** 现有 `name` 字段后端是 `String`（JSON 文件存储），无长度限制 | 改造 1 不需后端 schema 变更 | 仅前端去掉 `maxlength="60"` |
| **新发现：** `list_bookmarks` 后端已按 current video 自动过滤（`ipc/bookmark.rs:70`） | 改造 4 工作量小于预期 | 前端只需在视频切换事件后重调 `listBookmarks()` |

---

## BLOCKER 解决 — 后端契约（解 BLOCKER 1）

### 现有后端事实（基于 `src-tauri/src/ipc/bookmark.rs` 165 行）

```rust
pub struct BookmarkEntry {
    pub id: String,
    pub name: String,        // ← 无长度限制，serde_json 序列化到 bookmarks.json
    pub video: String,       // ← 视频路径，已经存在
    pub position: f64,
    pub created_at: u64,
}
```

- 存储路径：`{app_data_dir}/bookmarks.json`（不是 SQLite）
- `name: String` Rust 类型 → JSON 字符串，**无长度限制**
- `list_bookmarks` 后端代码 line 70 已用 `filter(|b| b.video == video_path)` 过滤当前视频

### 改造 1 落地（不增 `content` 字段）

**结论：复用现有 `name` 字段作为笔记内容载体，不新增 `content` 字段。**

理由：
- 后端无字段长度约束，复用零成本
- 新增字段需 backfill 旧数据
- "name vs content" 的语义切分对 5 用户验证阶段没价值，先复用，确实有问题再切

**前端动作：**
- `frontend/src/components/BookmarkView.vue:9` 去掉 `maxlength="60"`（或改 `maxlength="2000"` 防极端长输入）
- `<input>` 改为 `<textarea>`，`rows="2"`，CSS `resize: vertical`
- `frontend/src/api/player.ts:168-174` 不动 type

### 改造 4 落地

- `listBookmarks()` 不动，后端已过滤
- 前端切视频事件钩子调 `listBookmarks()` 重载列表
- 切换瞬间显示 loading 状态（避免空列表闪烁）

---

## 视觉规格（解 BLOCKER 2）

> 容器：PluginPopup 420×380（不是 280px sidebar）。所有尺寸已按 420 重核。

### 字体分工

| 元素 | 字体族 | 大小 | letter-spacing | 字重 | text-transform |
|------|--------|------|----------------|------|----------------|
| Header "NOTES" | Rajdhani | 11px | 0.08em | 600 | uppercase |
| Header "· 当前视频" | DM Sans | 12px | 0 | 400 | none |
| Header "N 条" 计数 | JetBrains Mono | 11px | 0.04em | 500 | none |
| 位置 chip "00:32:11" | JetBrains Mono | 12px tabular-nums | 0.04em | 500 | none |
| chip 状态字 "PAUSED" | Rajdhani | 10px | 0.08em | 600 | uppercase |
| textarea placeholder | DM Sans | 13px | 0 | 400 | none |
| textarea content | DM Sans | 14px | 0 | 400 | none |
| 提示文字 "Enter 保存 · Esc 取消" | JetBrains Mono | 10px | 0.04em | 400 | none |
| 列表项 timecode "00:14:32" | JetBrains Mono | 11px tabular-nums | 0.04em | 500 | none |
| 列表项内容 | DM Sans | 12px | 0 | 400 | none |
| 添加按钮 "ADD" | Rajdhani | 11px | 0.06em | 600 | uppercase |
| 导出按钮 "EXPORT" | Rajdhani | 11px | 0.06em | 600 | uppercase |
| 空状态提示 | DM Sans | 12px | 0 | 400 | none |
| 错误文案 | DM Sans | 11px | 0 | 400 | none |

### 颜色态表

| 元素 | 默认 | hover | focus | active | disabled |
|------|------|-------|-------|--------|----------|
| Header 背景 | transparent | — | — | — | — |
| Header "NOTES" 文字 | `var(--text-primary)` (#E8ECF1) | — | — | — | — |
| Header "· 当前视频" | `var(--text-muted)` (#6B7280) | — | — | — | — |
| chip 容器（默认） | `rgba(0,229,255,0.10)` bg + `rgba(0,229,255,0.20)` border | — | — | — | — |
| chip 容器（PAUSED） | `rgba(255,0,110,0.10)` bg + `rgba(255,0,110,0.20)` border | — | — | — | — |
| chip 时间码文字 | `var(--accent-cyan)` (#00E5FF) | — | — | — | — |
| chip PAUSED 文字 | `var(--accent-magenta)` (#FF006E) | — | — | — | — |
| textarea border | `var(--border-subtle)` (rgba(255,255,255,0.06)) | `rgba(255,255,255,0.12)` | `var(--accent-cyan)` + 0 0 8px rgba(0,229,255,0.20) glow | — | `var(--border-subtle)` opacity 0.5 |
| textarea bg | `var(--bg-elevated)` (#1A1A24) | — | — | — | `var(--bg-elevated)` opacity 0.5 |
| textarea content 文字 | `var(--text-primary)` | — | — | — | `var(--text-muted)` |
| textarea placeholder | `var(--text-muted)` | — | — | — | — |
| 添加按钮（secondary 风格） | transparent + `var(--accent-cyan)` border + cyan 文字 | bg `rgba(0,229,255,0.08)` | + 0 0 12px rgba(0,229,255,0.30) | bg `rgba(0,229,255,0.16)` | opacity 0.5 |
| 导出按钮（ghost 风格） | transparent + `var(--text-muted)` | `var(--text-primary)` 文字 + cyan icon | + 0 0 8px rgba(0,229,255,0.20) | — | opacity 0.5 |
| 列表项 border | `var(--border-subtle)` | `var(--accent-cyan)` | — | — | — |
| 列表项 bg | `var(--bg-surface)` (#12121A) | `var(--bg-surface)` (无变化，靠 border 反馈) | — | — | — |
| 列表项 timecode | `var(--text-primary)` | — | — | — | — |
| 列表项内容 | `var(--text-primary)` | — | — | — | — |
| 删除 × | `var(--text-muted)` | `var(--accent-magenta)` | — | — | — |
| Toast "已复制 N 条"（瞬时态） | `var(--accent-cyan)` 文字 + `rgba(0,229,255,0.08)` bg | — | — | — | — |

### Spacing & Radius

| 元素 | radius | padding | gap |
|------|--------|---------|-----|
| 面板根容器 | `lg` 12px（PluginPopup 已定） | `lg` 16px | — |
| 三段式行间距 | — | — | `md` 12px |
| Header | — | `sm` 8px 0 | `xs` 4px |
| chip | `full` 9999px | `xs` 4px `md` 12px | `xs` 4px |
| textarea | `sm` 4px | `sm` 8px `md` 12px | — |
| textarea 与按钮行间距 | — | — | `sm` 8px |
| 按钮（添加 / 导出） | `sm` 4px | `xs` 4px `md` 12px | — |
| 列表 | — | — | `2xs` 2px |
| 列表项 | `md` 8px | `sm` 7px `md` 10px | — |

### 动效

| 触发 | 动画 | 时长 | easing |
|------|------|------|--------|
| textarea focus | border 变 cyan + glow 渐入 | 150ms | ease-out |
| chip 切 PAUSED | bg/border/文字色 cross-fade | 200ms | ease-in-out |
| 新增列表项 | opacity 0→1 + translateY(4px→0) | 150ms | ease-out |
| 删除列表项 | opacity 1→0 + height 收缩 | 120ms | ease-in |
| 列表项 hover | border 渐变到 cyan | 150ms | ease-out |
| Toast 出现 | opacity 0→1 | 100ms | ease-out |
| Toast 消失 | opacity 1→0 | 200ms | ease-in (2s 后触发) |
| 自动暂停回退 | chip cyan→magenta→cyan 不闪烁，单向 cross-fade | 200ms | ease-in-out |

**不引入** Title Sweep / Breathing Pulse（DESIGN.md 定义但本组件不需要——这是工具区，不是品牌区）。

### Icon 政策

**所有 icon 使用 SVG，不用 emoji。** 推荐 Phosphor Icons (line 风格) 或 Lucide。

| 用途 | Icon 名 | size | color |
|------|---------|------|-------|
| 位置 chip 前缀 | `bookmark-simple` 或 `map-pin` | 12px | currentColor |
| 列表项前 ▶ | `play` | 10px | `var(--text-muted)` → hover `var(--accent-cyan)` |
| 删除 × | `x` | 14px | currentColor |
| 导出按钮 | `clipboard-text` 或 `download-simple` | 14px | currentColor |
| 添加按钮 | 无 icon，纯文字 ADD | — | — |

---

## A11y 段（解 BLOCKER 3）

### 语义化

| 元素 | 当前 HTML | 改造后 HTML | 理由 |
|------|----------|------------|------|
| 列表项 | `<div @click>` (BookmarkView:32) | `<button type="button">` | div+onClick 键盘不可达，screen reader 不识别为可点击 |
| 删除按钮 | `<button title="删除书签">` (line 39) | `<button type="button" aria-label="删除笔记 {bm.name}">` | title 不是 a11y label，部分 SR 不读 |
| textarea | 无 label | `<label for="note-input">` + `<textarea id="note-input" aria-label="笔记内容">` | 必需 |
| chip 时间 | `<span>` | `<span role="status" aria-live="polite">` 当状态变 PAUSED 时通报 | 让 SR 用户感知自动暂停 |
| 添加按钮 | `<button>` | `<button type="button" aria-disabled={!newName.trim()}>` | disabled 状态对 SR 明确 |
| 列表容器 | `<div>` | `<ul>` + `<li>` | 列表语义 |

### 键盘可达性

| 键 | 焦点位置 | 动作 |
|----|----------|------|
| `Tab` | textarea ↔ ADD 按钮 ↔ EXPORT 按钮 ↔ 列表第一项 ↔ ...（连续 Tab 顺次） | 标准 |
| `Tab` 进入列表后 | 列表项之间 | 同 Tab，每个 `<button>` 单独 focusable |
| `Enter` 焦点在列表项 | — | 触发 `seekToNote`（=点击效果） |
| `Delete` 焦点在列表项 | — | 删除该项（需确认 dialog） |
| `Esc` 焦点在 textarea | — | 清空输入 + blur |

### 焦点环

- 全局规则：禁止 `outline: none` without replacement
- textarea focus 时：`border-color: var(--accent-cyan)` + `box-shadow: 0 0 0 2px rgba(0,229,255,0.20)`（替代浏览器 outline）
- 列表项 / 按钮 focus 时：`box-shadow: 0 0 0 2px var(--accent-cyan)`（visible focus ring）
- 必须 `outline: 2px solid transparent; outline-offset: 2px` 以保留 Windows 高对比度模式可见性

### 对比度

| 文字 vs 背景 | 比值 | WCAG |
|-------------|------|------|
| `#E8ECF1` text-primary on `#12121A` bg-surface | ≈14.5:1 | AAA |
| `#6B7280` text-muted on `#12121A` bg-surface | ≈4.1:1 | AA (大字 AAA) |
| `#00E5FF` accent-cyan on `#12121A` bg-surface | ≈10.2:1 | AAA |
| `#FF006E` accent-magenta on `#12121A` bg-surface | ≈4.6:1 | AA |

所有当前色板均 ≥ AA。**不需要新增颜色。**

### Screen Reader 通报关键节点

| 时机 | 通报内容 |
|------|---------|
| 视频暂停（因 textarea focus 触发） | "视频已暂停，正在记笔记" |
| 笔记添加成功 | "已添加笔记，当前共 N 条" |
| 笔记删除成功 | "已删除笔记 {name}" |
| 点击列表项跳回放 | "跳到 {timecode} 并继续播放" |
| 导出复制成功 | "已复制 N 条笔记到剪贴板" |

实现：用一个 visually-hidden `<div role="status" aria-live="polite">` 容器，更新 textContent 触发通报。

---

## 热键定义（解 BLOCKER 4）

### 现状

- App.vue 当前**没有任何全局键盘事件监听**（`grep keydown/keyup` 无命中）
- 现有交互全靠鼠标 + `@keyup.enter` 单点（SubtitleSearch.vue:8, BookmarkView.vue:10）
- 这意味着引入热键系统是**新基础设施**，不仅是 NoteView 范围

### 本周热键最小集（仅 NoteView 域）

| 键 | 上下文 | 动作 | 实现位置 |
|----|--------|------|---------|
| `N` | 全局，且 NoteView 面板已开 | 焦点跳到 textarea，自动暂停 | App.vue 全局监听 |
| `Enter` | 焦点在 textarea | 保存当前笔记（与现行 `@keyup.enter` 一致） | NoteView 组件 |
| `Shift+Enter` | 焦点在 textarea | 换行（多行笔记） | textarea 默认行为，不拦截 |
| `Esc` | 焦点在 textarea | 清空 + blur + 恢复原播放状态 | NoteView 组件 |
| `⌘Enter` (Mac) / `Ctrl+Enter` (Win) | 焦点在 textarea | 强制保存（即使内容为空也不保存，但跳出焦点） | NoteView 组件 |

### 冲突检查

- 当前播放器**没有 `N` 热键占用**（也没有 Space / 方向键热键 — 整个项目没接热键）
- 一旦本周引入 `N`，未来扩展到 Space 暂停 / ↑↓ 音量等，需保留 `N` 给笔记
- 提示用户：在 textarea **focus 内**所有热键应仅作用于 textarea，**不冒泡**到全局播放器热键。实现时 textarea 容器 `@keydown.stop` 阻断冒泡。

### 视觉提示

textarea 下方一行 hint，10px JetBrains Mono muted：

```
ENTER 保存 · SHIFT+ENTER 换行 · ESC 取消
```

首次显示后用户可在设置里关闭（hint 显隐状态持久化到 localStorage）。

### 全局热键基础设施

新建文件 `frontend/src/composables/useHotkeys.ts`，最小实现：
- `useHotkey(key, handler, options?)` composable
- 支持 `enabled` ref 切换（NoteView 关闭时禁用 `N`）
- 支持 `target: 'global' | 'element'`
- 自动 cleanup on unmounted

**SCOPE 注：** 本周只接 `N` 一个热键。其它热键（Space 暂停 / 方向键 seek）属下一轮扩展。

---

## BLOCKER 解除状态

| BLOCKER | 状态 | 落点 |
|---------|------|------|
| 1. `BookmarkEntry.name` 长度限制 | ✅ 解 | 上文 §BLOCKER 解决 — 后端契约 |
| 2. 字体分工 + 颜色态 | ✅ 解 | 上文 §视觉规格 |
| 3. A11y 全空白 | ✅ 解 | 上文 §A11y 段 |
| 4. 热键定义 | ✅ 解 | 上文 §热键定义 |

WARN 项中尚未解决的：
- 中英混排规则补充到 DESIGN.md（建议本周 PR 顺手做）
- 导出按钮位置（已锁 Header 右侧，见 §视觉规格 §Icon 政策）
- PAUSED 视觉信号（已锁 chip 变 magenta，见 §视觉规格 §颜色态表）
- 无视频时面板状态（textarea disabled，chip 显 `--:--:--`，列表显 "请先打开视频"）
- 导出成功 toast（已锁，见 §视觉规格 §动效 + §颜色态表）
- 切视频时未保存笔记保护（建议：textarea 非空时弹 confirm，可放 §改造 4 落地后做小迭代）
- 视频结尾 t-3s 边界（建议：position - 3 < 0 → 跳 0；其他情况不调整）

---

## 准备进入 /plan-eng-review

可以进 eng-review 了。预期 eng-review 关注点：
- 自动暂停"原状态快照"存哪（localStorage / Pinia / Vue ref）
- 视频切换事件钩子的位置（已有 player event bus？还是 watch playlist.current_index？）
- `useHotkeys.ts` composable 与 Tauri global shortcut 的边界（前者只在 webview focus 时生效）
- 列表 enter/exit 动画与 Vue `<TransitionGroup>` 的集成成本
- 5 用户验证的埋点策略（plugin event bus 已有 `bookmark:add` / `bookmark:remove` / `bookmark:list`，可直接复用作笔记埋点 — 见 `src-tauri/src/ipc/bookmark.rs:73,126`）
