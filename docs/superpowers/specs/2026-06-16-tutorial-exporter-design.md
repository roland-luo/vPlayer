# vPlayer Tutorial Exporter 设计文档

> 日期：2026-06-16  
> 状态：DRAFT — 等待实现计划  
> 上游依据：/superpowers:brainstorming 会话（2026-06-16）

---

## 1. 目标

让技术教程学习者能把一次观看中产生的笔记，一键整理成可以直接贴到 GitHub / Gist / 博客的 Markdown 文章。

## 2. 范围

### 2.1 做这个

- 选择多条笔记 → 导出为 `.md` 文件
- 自动按章节或时间顺序排列笔记
- 每条笔记附带：可点击时间戳、可选截图、笔记正文
- 保留笔记中的 Markdown 代码块格式
- 导出文件保存到用户指定目录

### 2.2 不做这个

- 不自动 AI 生成摘要
- 不直接上传到 GitHub / 博客平台
- 不输出 PDF / Word 等富格式
- 不做复杂的富文本编辑

## 3. 用户流程

1. 用户看视频，使用现有 Notes 面板记录时间戳笔记
2. 在 Notes 面板点击新增「Export」按钮
3. 弹出 Tutorial Exporter 弹窗：
   - 显示当前视频所有笔记列表
   - 支持全选 / 取消 / 单条勾选
   - 输入文件名
     - 默认值为 `{视频文件名}-notes.md`
     - 校验：非空、不包含路径分隔符、以 `.md` 结尾
   - 开关：是否插入截图、是否按章节分组
4. 点击「Export .md」→ 弹出系统保存目录对话框
5. 后端生成 Markdown 文件和 `assets/` 目录
6. 前端显示导出成功 Toast，提供「打开目录」按钮

## 4. UI 设计

### 4.1 入口

- 位置：现有 Notes 面板（`BookmarkView.vue`）头部右侧
- 样式：Ghost 按钮，图标 `download-simple`，文字「EXPORT」

### 4.2 弹窗容器

复用现有 `PluginPopup`，尺寸 420×520（比 Notes 面板稍高以容纳勾选列表）。

### 4.3 弹窗内部结构

```
┌─────────────────────────────────────┐
│ EXPORT TUTORIAL           12 notes  │
├─────────────────────────────────────┤
│ File name                           │
│ [rust-async-tutorial.md        ]    │
├─────────────────────────────────────┤
│ [✓] Include screenshots             │
│ [✓] Group by chapters               │
├─────────────────────────────────────┤
│ Select notes                        │
│ [✓] 00:03:42 创建 tokio runtime...  │
│ [✓] 00:08:15 await 和 block_on...   │
│ [ ] 00:12:30                        │
│ ...                                 │
├─────────────────────────────────────┤
│                       [Cancel] [Export]│
└─────────────────────────────────────┘
```

### 4.4 视觉规格

遵循 DESIGN.md：

- 标题：Rajdhani 11px uppercase，letter-spacing 0.08em
- 时间码：JetBrains Mono 11px tabular-nums
- 正文：DM Sans 12px
- 按钮：Secondary 风格（Export），Ghost 风格（Cancel）
- 列表项：bg-surface #12121A，hover border cyan
- 输入框：bg-elevated #1A1A24，focus cyan glow

## 5. 导出格式

### 5.1 文件结构

```
output-dir/
├── rust-async-tutorial.md
└── assets/
    ├── 00_03_42.png
    ├── 00_08_15.png
    └── ...
```

### 5.2 Markdown 样例

```markdown
# Rust Async 教程笔记

> 源视频：rust-async-tutorial.mp4
> 导出时间：2026-06-16

---

## 01. Tokio Runtime

### [00:03:42](vplayer://open?file=...&t=222) 创建 tokio runtime 的两种写法

一种是使用 `#[tokio::main]` 宏：

```rust
#[tokio::main]
async fn main() {
    // ...
}
```

另一种是手动创建 Runtime：

```rust
let rt = tokio::runtime::Runtime::new().unwrap();
rt.block_on(async {
    // ...
});
```

![screenshot](assets/00_03_42.png)

### [00:08:15](vplayer://open?file=...&t=495) await 和 block_on 的区别

- `await` 必须在 async 函数内使用
- `block_on` 会阻塞当前线程直到 future 完成

---

## 02. Spawn Tasks

...
```

### 5.3 时间戳链接格式

```
vplayer://open?file=<url-encoded-path>&t=<seconds>
```

- `file`：视频绝对路径，URL encode
- `t`：笔记时间戳（秒，整数）
- 当前阶段仅作为文本链接，未来播放器注册 URI scheme 后可直接跳转

### 5.4 章节分组规则

- 开启「Group by chapters」且视频有章节数据：按章节标题作为 `##` 二级标题，笔记归入对应章节
- 无章节数据：按时间线平铺，每条笔记为 `###` 三级标题
- 章节时间范围外的笔记：统一放入「未分类」小节

## 6. 技术架构

### 6.1 后端

新增 Tauri command：`export_notes_to_markdown`

输入 payload：

```rust
pub struct ExportNotesRequest {
    pub video_path: String,
    pub note_ids: Vec<String>,
    pub output_dir: String,
    pub filename: String,
    pub include_screenshots: bool,
    pub group_by_chapters: bool,
}
```

输出 payload：

```rust
pub struct ExportNotesResult {
    pub markdown_path: String,
    pub assets_dir: String,
    pub note_count: usize,
    pub screenshot_count: usize,
}
```

处理流程：

1. 校验 `video_path` 存在且可读
2. 调用现有 `list_bookmarks(video_path)` 获取笔记
3. 按 `note_ids` 过滤用户选中的笔记
4. 如果 `include_screenshots`：
   - 对每个笔记时间点调用现有 `capture_screenshot`
   - 图片保存到 `{output_dir}/assets/{hh_mm_ss}.png`
5. 如果 `group_by_chapters`：
   - 调用现有 `list_chapters(video_path)`
   - 按章节分组
6. 渲染 Markdown 字符串
7. 创建 `{output_dir}/assets/` 目录（若不存在；若已存在则直接复用）
8. 写入 `{output_dir}/{filename}`
9. 返回结果

### 6.2 前端

新增文件：

- `frontend/src/components/TutorialExporter.vue` — 导出弹窗组件
- `frontend/src/api/tutorialExport.ts` — IPC 调用封装

改动文件：

- `frontend/src/components/BookmarkView.vue` — 头部新增 Export 按钮
- `frontend/src/App.vue` — 注册弹窗打开/关闭状态

### 6.3 复用能力

| 能力 | 来源 |
|------|------|
| 笔记读取 | `src-tauri/src/ipc/bookmark.rs` |
| 截图 | `src-tauri/src/ipc/commands.rs` / `builtin/screenshot.rs` |
| 章节 | `src-tauri/src/ipc/chapter.rs` |
| 弹窗容器 | `frontend/src/components/PluginPopup.vue` |
| 文件对话框 | `rfd::FileDialog` |

## 7. 边界情况

| 场景 | 处理 |
|------|------|
| 笔记列表为空 | Export 按钮 disabled |
| 用户取消全部勾选 | Export 按钮 disabled |
| 视频未加载 | 弹窗显示「请先打开视频」|
| 单条截图失败 | 跳过该图片，Toast 提示失败数量 |
| 无章节数据但开启分组 | 自动降级为时间线平铺 |
| 文件名已存在 | 弹出覆盖确认对话框 |
| 输出目录不可写 | 返回 `FILE_PERMISSION_DENIED`，保留弹窗状态 |
| 时间戳为 0 或越界 | `t=max(0, position)`，截图使用 clamp 后的时间点 |
| 笔记内容含 Markdown 特殊字符 | 按原样输出，不转义（信任用户输入）|

## 8. 错误处理

后端错误统一返回 `Result<ExportNotesResult, String>`，错误消息用户可读。

错误码前缀：`EXPORT_*`

| 错误码 | 含义 | 前端处理 |
|--------|------|----------|
| `EXPORT_FILE_NOT_FOUND` | 视频文件不存在 | Toast + 关闭弹窗 |
| `EXPORT_PERMISSION_DENIED` | 输出目录无写入权限 | Toast + 保留弹窗，可重选目录 |
| `EXPORT_WRITE_FAILED` | 文件写入失败 | Toast + 保留弹窗 |
| `EXPORT_INVALID_FILENAME` | 文件名为空、包含 `/` 或 `\`、或无 `.md` 后缀 | 输入框边框变 magenta，提示修改 |
| `EXPORT_NO_NOTES_SELECTED` | 未选择任何笔记 | Export 按钮 disabled，避免触发 |

## 9. 验收标准

- [ ] Notes 面板出现 Export 按钮
- [ ] 弹窗能正确列出当前视频所有笔记
- [ ] 勾选笔记后点击 Export 弹出保存目录对话框
- [ ] 导出文件包含正确的 Markdown 结构和时间戳链接
- [ ] 开启截图后 `assets/` 目录生成对应图片
- [ ] 开启章节分组后输出按章节组织
- [ ] 截图失败时不中断整体导出
- [ ] 输出目录不可写时给出明确错误提示

## 10. 决策记录

| 决策 | 选择 | 理由 |
|------|------|------|
| 导出格式 | Markdown | 用户目标平台是 GitHub/Gist/博客，Markdown 最通用 |
| 截图存储 | `assets/` 子目录 + 相对路径 | 方便直接迁移到博客或 GitHub |
| 章节分组 | 复用 Chapter 插件数据 | 避免重新解析 ffprobe |
| 时间戳链接 | `vplayer://` URI scheme | 为未来的播放器跳转预留接口 |
| 不自动上传 | 仅本地导出 | 控制范围，避免引入平台 OAuth |
| 不 AI 生成摘要 | 完全基于用户笔记 | 控制范围，保证内容可控 |

## 11. 开放问题

1. 是否需要支持导出为纯文本（无 Markdown）？
2. 是否需要记住上次的导出目录？
3. 时间戳链接未来注册 URI scheme 的具体方案？
4. 是否需要在导出前预览 Markdown？

---

*本 spec 由 /superpowers:brainstorming 生成，下一步调用 /superpowers:writing-plans 制定实现计划。*
