# Plan Design Review: Notes View (Variant A)

> 日期：2026-05-12
> 上游计划：`docs/notes-view-plan.md`
> 选定版式：Variant A（顶部输入 + 下方列表，改造现 BookmarkView）
> 参考标准：`DESIGN.md` (Neo-Industrial Cyber-Minimal)
> 执行者：plan-design-review skill
> 验证范围：审 plan 的设计完备性，不审代码

---

## 0. TL;DR

| 等级 | 初审 | 复审 (2026-05-12 second pass) |
|------|------|-------------------------------|
| 🟥 BLOCKER | 3 + 1⚠️ | **0**（全部已在 plan §BLOCKER 解除状态 中解决） |
| 🟧 WARN | 7 | 4（其余 3 项已通过视觉规格 / icon 政策 / 颜色态表落地） |
| 🟩 PASS | 4 | 8 |

**核心结论：** Variant A 方向正确，但 plan 与 DESIGN.md 之间存在"视觉规格未对齐"的空白带。3 项 BLOCKER 全部围绕**"现有数据模型够不够支撑改造"**——只要后端契约不变（`BookmarkEntry.name` 60 char 限制 / `listBookmarks` 无视频过滤），Variant A 无法落地。

**复审更新（2026-05-12）：** 4 项 BLOCKER 全部解决，且后端实际事实**比初审假设更友好**——`name` 字段 Rust String 无长度限制、`list_bookmarks` 后端已自动按 current video 过滤。改造工作量比预期小。

---

## 1. 数据契约前置检查（PRE-AUDIT）

读 `frontend/src/api/player.ts:168-186` 后确认：

```ts
export type BookmarkEntry = {
  id: string;
  name: string;          // ← 当前 input maxlength=60，要变 textarea 必须松绑
  video: string;         // ← 已有视频绑定字段
  position: number;
  created_at: number;
};

export async function listBookmarks(): Promise<BookmarkEntry[]>;
//                                  ↑ 无 video 过滤参数，全局列表
export async function addBookmark(name: string): Promise<BookmarkEntry>;
//                                ↑ 只传 name，position 由后端 from 当前播放状态
```

**好消息：** `video` 字段已存在 → 改造 4 后端 schema 不用动。
**坏消息：** `name` 字段当前是"名称"心智模型且前端写死 60 字符 → 改造 1 需要后端确认 `name` 是否能装 ≥500 字符长文本，否则需要新增 `content` 字段。

---

## 2. Pass 1 — 信息架构（Info Architecture）

### 🟩 PASS · 三段式层级清晰

Variant A 的 `header → input zone → list` 三段式与 DESIGN.md 的 Top HUD / 中部内容 / 控制区结构同构。

### 🟧 WARN · 中英混排规则未定义

Variant A 头部预览：`NOTES · 当前视频`。
- "NOTES" 是 Rajdhani uppercase（DESIGN.md §Typography "Display/Hero"）
- "当前视频" 是 DM Sans 中文 body

DESIGN.md 没有定义 **中英混排时分隔符、字号比例、垂直基线对齐**。
若不定义，每个 UI 组件都会出现"英文标题 + 中文 metadata"组合，风格漂移。

**Action：** plan 里应明确"中英混排"规则，建议加到 DESIGN.md 的 Typography 段：
- 标题区：`<英文 Rajdhani UPPERCASE> · <DM Sans 中文 12px muted>`
- 二者之间用 `·` 分隔，前后空格

### 🟧 WARN · 导出按钮在 header 还是 list footer 不明

Variant A 把 📋 放在 header 右侧。
plan §改造 5 写"列表底部一个'导出 .md'按钮"。
两处不一致。

**Action：** 锁定一处。建议 header 右侧（一直可见），与"导出 = 全局动作"心智模型一致。

---

## 3. Pass 2 — 状态完备性（States）

现有 BookmarkView 已实现 4 个状态（loading / error / empty / list），改造后新增 4 个状态 plan 未覆盖：

| 缺失状态 | 何时出现 | plan 是否提及 |
|---------|---------|---------------|
| **写入中**（textarea focus + video paused） | 改造 2 触发后 | ❌ |
| **保存 inflight**（点添加 → 网络/IPC 往返） | 改造 1 添加流程 | ❌ |
| **导出 inflight + success toast** | 改造 5 触发 | ❌ |
| **切视频 reload 中**（列表清空 → 新视频列表载入） | 改造 4 触发 | ❌ |
| **无视频打开时**（用户先打开笔记面板，再选视频） | 起始场景 | ❌（应禁用 textarea + chip 显示 `--:--:--`） |

### 🟧 WARN · "写入中"状态缺视觉信号

改造 2 说"focus 触发 pause"，但**用户怎么知道**已经暂停了？
- 当前 player UI 是否会自动反馈暂停按钮变化？需要 plan 明确"自动暂停时，chip 颜色变化 / 加 'PAUSED' 字样"
- DESIGN.md `#FF006E` Alert Magenta 适合这种"系统介入中"信号

**Action：** plan §改造 2 增补：「focus 后 chip 变 magenta 边框 + 文字加 'PAUSED'，资料保存或失焦后恢复 cyan」。

### 🟧 WARN · 无视频时面板状态

用户可能先打开笔记面板再选视频。当前 plan 没定义这一刻：
- textarea 是否禁用？
- chip 显示什么？`--:--:--`？还是隐藏？
- 列表是否显示 "请先打开视频" 的引导？

**Action：** 加一条 empty-no-video 状态规格。

---

## 4. Pass 3 — 用户旅程（Journey）

按 office-hours doc 的"30 分钟真实学习任务"假想旅程：

```
打开 vPlayer → 选视频 → 看到面板（空列表）→
看 5 分钟 → 想记一笔 → ???? → 写完 → 按 ???? 保存 →
继续看 → 想记第二笔 → ...  → 看完 → 想回顾 → 点列表 → 自动跳回放 →
看完任务 → ???? → 拿到 markdown 给我
```

### 🟥 BLOCKER · "??? 怎么开始写"——键盘热键缺失

- 当前 plan 假设用户**鼠标移到面板 → 点 textarea**
- 看视频用户**80% 时间手在键盘上**（暂停 = Space、音量 = ↑↓）
- 没有"按 N 直接进入笔记输入"快捷键，用户必须中断观看主动找笔记面板
- 这与 office-hours 验证目标"≥5 条笔记"直接相关——摩擦力越低，写笔记越多

**Action：** plan 必须定义热键 (建议 `N` 进入笔记、`Esc` 取消、`Enter` 保存)。这是**视觉决策**因为热键提示文字位置会影响 UI 布局（例如 textarea 下面要不要写"Enter 保存 · Esc 取消"hint）。

### 🟧 WARN · 导出成功后无反馈

改造 5 "复制到剪贴板"——但用户**怎么知道复制成功了**？
- 一般做法：toast 通知 "已复制 3 条笔记" 2s 自动消失
- DESIGN.md 没有定义 toast 组件

**Action：** plan 锁定"复制成功 → 顶部 chip 区域瞬时变 cyan + 文字"已复制 N 条" + 2s 后恢复"。

### 🟧 WARN · 5 用户验证的"埋点"完全没规划

plan 末尾说"5-user 验证"，但 4 个指标（笔记条数、回跳次数、任务完成率、是否愿意继续使用）的**采集方式**没说：
- 笔记条数 / 回跳次数：可从 `listBookmarks()` 返回长度 + 一个新增的 `seek_from_bookmark` 计数器
- 任务完成率 / 愿意继续：得是访谈时人工填表

**Action：** 这是 plan-eng-review 的事，但 plan-design-review 应至少要求 plan 把"在哪里能看到当次会话的笔记条数 / 回跳次数"列入需求——通常 = header 那条"NOTES · 7 条" 元数据。

---

## 5. Pass 4 — AI Slop 风险

DESIGN.md §Aesthetic Direction 已明确禁止 "purple gradients / generic icon grids / bubbly border-radius / 居中营销布局"。

### 🟩 PASS · Variant A 没有 AI 视觉陈词滥调

ASCII 预览中无渐变、无 emoji 滥用、无圆角溢出。

### 🟧 WARN · emoji 使用政策未明（📋 📍 ▶ ×）

plan 和 Variant A 预览使用了 4 个 emoji：
- 📋 导出
- 📍 当前位置 chip
- ▶ 列表项前置
- × 删除

DESIGN.md 没说"是用 emoji 还是 SVG icon"。在 mono 字体环境里 emoji 会破坏 tabular-nums 对齐。

**Action：** plan 必须二选一：
- A: 全用 emoji（最快上线，但视觉劣化）
- B: 全用线性 SVG icon（建议）：Phosphor / Tabler 库，1.5px stroke，颜色 `currentColor`

**推荐 B。**

---

## 6. Pass 5 — Design System 对齐

### 🟥 BLOCKER · 字体分工 plan 没指定

每个元素该用哪个字体（Rajdhani/DM Sans/JetBrains Mono），plan 完全没说。
DESIGN.md §Typography 给了规则但没下沉到 NoteView 各元素：

| 元素 | 应当字体（按 DESIGN.md 推导） | plan 是否说 |
|------|----------------------------|------------|
| header "NOTES" | Rajdhani UPPERCASE 11px ls 0.08em | ❌ |
| header "当前视频" | DM Sans 12px muted | ❌ |
| chip "📍 00:32:11" | JetBrains Mono 11px tabular-nums | ❌ |
| chip "PAUSED" 状态字（如果有） | Rajdhani UPPERCASE 10px ls 0.08em | ❌ |
| textarea placeholder | DM Sans 14px muted | ❌ |
| textarea content | DM Sans 14px primary | ❌ |
| 列表项 timecode | JetBrains Mono 11px tabular-nums | ❌ |
| 列表项内容 | DM Sans 12px primary | ❌ |
| "添加" / "导出" 按钮 | Rajdhani UPPERCASE 11px ls 0.06em weight 600 | ❌ |

**Action：** 上表搬进 plan §视觉规格段落，逐元素锁定。

### 🟥 BLOCKER · 颜色 / 边框 / 焦点态 plan 没指定

DESIGN.md §Color 给了 `#00E5FF` cyan + `#FF006E` magenta + 三层中性灰。但 plan 没说：

| 元素 | 默认 | hover | focus | disabled |
|------|------|-------|-------|----------|
| textarea border | `rgba(255,255,255,0.06)` | ? | cyan | muted |
| 添加按钮 | primary cyan? secondary border? | ? | ? | opacity 0.5 |
| 列表项 border | subtle | cyan border | — | — |
| 删除 × | muted | magenta | — | — |
| 导出按钮 | secondary border? ghost? | ? | ? | — |

**Action：** plan 增补一张"颜色态表"。

### 🟧 WARN · 圆角 / spacing 没引用 DESIGN.md 4px scale

plan 完全没写具体 px 值。
DESIGN.md 给了 `radius-sm 4 / md 8 / lg 12`，`spacing 2xs 2 / xs 4 / sm 8 / md 12 / lg 16`。

**Action：** plan 加一段"组件 spacing & radius"，建议：
- textarea: `radius-sm 4px`，padding `sm 8px`
- chip: `radius-full 9999`，padding `xs 4px md 12px`
- 列表项: `radius-md 8px` (与现 BookmarkView 一致)
- 面板间距: gap `sm 10px` (BookmarkView 现状)

### 🟧 WARN · 动效 plan 完全没提

DESIGN.md §Motion 定义了 5 种签名动效，plan 一种都没引用。
最相关的 2 项：

- **Title Sweep**：可能用于空状态引导文案（"暂无笔记 · 按 N 开始"）
- **Breathing Pulse**：可用于 chip 在录制态（PAUSED 时呼吸）

**Action：** plan 加一段"动效",至少声明：
- chip focus 时 200ms cyan border fade-in
- 新增列表项 entry: opacity 0→1 + translateY(4px→0) 150ms ease-out
- 删除列表项 exit: opacity 1→0 100ms ease-in

---

## 7. Pass 6 — 响应式 + 无障碍

### 🟥 BLOCKER · A11y 全空白

plan 全文没出现 aria / tab order / 对比度 / 键盘可达性。这是审查发现的最严重缺口之一。

最小要求：
- textarea: `aria-label="笔记内容（按 Enter 保存）"`
- chip: `role="status" aria-live="polite"` 当 PAUSED 状态切换时通报
- 列表项: `<button>` 而不是 `<div onClick>`（当前 BookmarkView 用了 `<div @click>` 是 a11y 倒退）
- 删除按钮: `aria-label="删除笔记"` (当前 BookmarkView 已用 `title=` 但 title 不是 a11y label)
- 焦点环: 可见的 `outline` 或 `box-shadow`（cyan glow 即可），不能 `outline: none`

**Action：** plan 加 §a11y 段。

### 🟧 WARN · 窄屏 / 缩放回退未定义

DESIGN.md 说 panel 280px。如果窗口窄到 <480px 怎么办？
- panel 是否可叠到视频上方？
- textarea 是否变 1 行？

本周可以暂不处理，但 plan 应标 "OUT_OF_SCOPE: <480px responsive"。

---

## 8. Pass 7 — 未决决策

### 🟥 BLOCKER · `BookmarkEntry.name` 60 char 限制

`frontend/src/components/BookmarkView.vue:9` 写死 `maxlength="60"`。
后端 schema 是否限制 60？plan 假设 ≥500 但**没有验证后端约束**。

如果后端 SQLite 列是 `TEXT` 无长度限制 → 改造 1 只需前端去掉 maxlength。
如果后端是 `VARCHAR(60)` → 改造 1 需要 backfill migration。

**Action：** plan-eng-review 必须先确认。本 review 把它从 WARN 升 BLOCKER 因为它决定**是否新增 `content` 字段**——这又决定 ts type 形状，进而决定前端 prop binding。

### 🟧 WARN · 切视频时旧笔记面板状态

plan 写"切视频列表自动重载"，但**正在写一半的笔记**怎么办？
- 丢弃？提示？保留草稿？

**Action：** plan 增补"切视频时若 textarea 非空，弹 confirm '当前笔记未保存，确定切换吗？'"。

### 🟧 WARN · t-3s 边界

plan 写 `Math.max(0, position - 3)`——但**视频结尾**呢？
position = duration - 1s 时 seek 到 duration - 4s 后视频是否继续播？

**Action：** plan §改造 3 加一行"若 position - 3 < 0 → 跳到 0；若 position 接近 duration → 不调整，按原 position 跳"。

### 🟧 WARN · 自动暂停 / 自动播放的"原状态快照"

改造 2 说"保存或失焦+空时自动 resume"，"保留用户原本的播放/暂停状态"。
- 原状态存哪？localStorage / Pinia / Vue ref？
- 跨 focus/blur 的状态机？

**Action：** plan-eng-review 接力。

### 🟩 PASS · 导出 markdown 格式定义足够

plan 写的 markdown 示例（视频名 + bullet 时间戳）格式简单，Notion/Obsidian 都吃。本周不需要再迭代。

### 🟩 PASS · "笔记编辑"明确不做

plan 把编辑放到验证之后，符合"5 改造点最小楔子"原则。本审查无意见。

---

## 9. 行动清单（按优先级）

### 🟥 进 `/plan-eng-review` 前必须解决（BLOCKER）

1. **确认 `BookmarkEntry.name` 后端长度限制** → 决定要不要新增 `content` 字段
2. **补字体分工表 + 颜色态表** → 加到 `notes-view-plan.md` 视觉规格段
3. **补 A11y 段** → aria-label / 键盘可达 / 焦点环
4. **补热键定义** → `N` 进入笔记输入、`Enter` 保存、`Esc` 取消、`⌘Enter` 强制保存（在多行模式下）

### 🟧 实现前应补（WARN）

5. 中英混排规则补充到 DESIGN.md
6. 导出按钮位置二选一（建议 header 右侧）
7. PAUSED 视觉信号定义
8. 无视频时的面板状态
9. 导出成功 toast / 反馈机制
10. 切视频时未保存笔记保护
11. 视频结尾 t-3s 边界
12. emoji vs SVG icon 政策（推荐 SVG）

### 🟩 可放行

- 三段式信息架构方向正确
- Variant A 无 AI slop 风险
- 导出 markdown 格式
- 笔记编辑明确不做

---

## 10. 审查结论

**Initial Verdict (2026-05-12 first pass)：CHANGES_REQUESTED** — 4 项 BLOCKER 阻碍 eng-review

**Revised Verdict (2026-05-12 second pass)：PASS** — 全部 4 项 BLOCKER 已在 `docs/notes-view-plan.md` §BLOCKER 解除状态 中解决。WARN 项已收敛到可在实现期间增量处理的程度。

### 复审证据

| BLOCKER | 解在何处 |
|---------|---------|
| 1. name 长度限制 | `notes-view-plan.md` §BLOCKER 解决 — 后端契约（含 `ipc/bookmark.rs` 代码引用） |
| 2. 字体 + 颜色 | `notes-view-plan.md` §视觉规格（14 行字体表 + 16 行颜色态表） |
| 3. A11y | `notes-view-plan.md` §A11y 段（语义化 + 键盘 + 焦点环 + 对比度 + SR 通报） |
| 4. 热键 | `notes-view-plan.md` §热键定义（含与现有播放器冲突检查） |
