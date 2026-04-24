# vPlayer 插件系统 — 功能状态

## 已实现功能 (Phase 1)

### 核心框架
| 功能 | 文件 | 状态 |
|------|------|------|
| `Plugin` trait（name/on_load/on_unload/on_event） | `src/plugin/mod.rs` | ✅ |
| `PluginManifest`（名称/版本/许可/UI描述） | `src/plugin/mod.rs` | ✅ |
| `PluginInstance`（manifest + 运行时状态） | `src/plugin/mod.rs` | ✅ |
| `PluginState`（Enabled / Disabled / Crashed） | `src/plugin/mod.rs` | ✅ |
| `PluginError` 枚举（Rejected/Panic/LoadFailed/等） | `src/plugin/mod.rs` | ✅ |
| `PluginEvent`（命名事件 + JSON payload） | `src/plugin/mod.rs` | ✅ |
| `PluginContext`（data_dir / config_path） | `src/plugin/mod.rs` | ✅ |
| `Permission` 声明式权限模型 | `src/plugin/mod.rs` | ✅ |
| `PluginBus` — 插件注册、事件分发、catch_unwind 隔离 | `src/plugin/bus.rs` | ✅ |
| `PluginRegistry` — 插件列表、开关、错误追踪 | `src/plugin/registry.rs` | ✅ |
| `PluginInfo` 含 UI 元数据（按钮/弹窗尺寸） | `src/plugin/registry.rs` | ✅ |
| `sandbox.rs` — 权限检查函数 + 单元测试 | `src/plugin/sandbox.rs` | ✅ |
| 启动时初始化（内置插件加载 + 外置插件目录扫描） | `src/plugin/mod.rs::init()` | ✅ |
| 外置插件 manifest.json 解析验证 | `src/plugin/loader.rs` | ✅ |
| 动态库加载（`.dylib/.so/.dll`）占位 | `src/plugin/loader.rs` | ⏳ Phase 2 |

### 内置插件
| 插件 | 功能 | 状态 |
|------|------|------|
| **Screenshot** — 截图功能 | `builtin/screenshot.rs` | ✅ |
| 　　・IPC 命令 `capture_screenshot` → ffmpeg 抽取帧 | `ipc/commands.rs` | ✅ |
| 　　・原生 Save As 对话框 (`rfd::FileDialog`) | `ipc/commands.rs` | ✅ |
| 　　・截图后通过 PluginBus 发送 `screenshot:captured` 事件 | `ipc/commands.rs` | ✅ |
| 　　・控制栏相机图标按钮 | `ControlBar.vue` | ✅ |
| 　　・保存成功/失败 Toast | `App.vue` | ✅ |
| **Subtitle Download** — 字幕下载 | `builtin/subtitle_download.rs` | ✅ |
| 　　・本地字幕搜索（同目录 .srt/.ass/.vtt/.sub） | `ipc/subtitle.rs` | ✅ |
| 　　・文件名自动匹配（支持语言标签 `video.en.srt`） | `ipc/subtitle.rs` | ✅ |
| 　　・多语言自动识别（中/英/日/韩/法/德/等） | `ipc/subtitle.rs` | ✅ |
| 　　・字幕下载（复制到视频同目录） | `ipc/subtitle.rs` | ✅ |
| 　　・搜索/下载事件跟踪计数 | `subtitle_download.rs` | ✅ |
| 　　・在线搜索 API | — | ❌ 未实现 |
| **Media Info** — 媒体信息 | `builtin/media_info.rs` | ✅ |
| 　　・ffprobe 解析视频元数据 | `ipc/mediainfo.rs` | ✅ |
| 　　・显示容器/时长/文件大小/总码率 | `MediaInfoView.vue` | ✅ |
| 　　・显示视频编码/分辨率/码率/帧率 | `MediaInfoView.vue` | ✅ |
| 　　・显示音频编码/声道/采样率 | `MediaInfoView.vue` | ✅ |
| **Playback Speed** — 播放速度 | `builtin/speed.rs` | ✅ |
| 　　・预设速度按钮（0.5x ~ 2.0x） | `PlaybackSpeedView.vue` | ✅ |
| 　　・滑块连续调节 | `PlaybackSpeedView.vue` | ✅ |
| 　　・HTML5 video.playbackRate 控制 | `PlayerView.vue` | ✅ |
| 　　・速度变更 Toast 反馈 | `App.vue` | ✅ |
| **Bookmark** — 书签 | `builtin/bookmark.rs` | ✅ |
| 　　・添加/删除书签（持久化 JSON 存储） | `ipc/bookmark.rs` | ✅ |
| 　　・点击书签跳转到对应时间点 | `BookmarkView.vue` | ✅ |
| 　　・加载中 / 空 / 错误状态 | `BookmarkView.vue` | ✅ |

### IPC / 前后端通信
| 功能 | 文件 | 状态 |
|------|------|------|
| `player:state_change` 事件 | `ipc/events.rs` | ✅ |
| `player:progress` 事件 | `ipc/events.rs` | ✅ |
| `video:error` 事件 | `ipc/events.rs` | ✅ |
| `app:fatal_error` 事件 | `ipc/events.rs` | ✅ |
| `plugin:state_changed` 事件 | `ipc/events.rs` | ✅ |
| `plugin:error` 事件 | `ipc/events.rs` | ✅ |
| `plugin:installed` 事件 | `ipc/events.rs` | ✅ |
| `list_plugins` / `toggle_plugin` / `get_plugin_detail` | `ipc/commands.rs` | ✅ |
| `search_subtitles` / `download_subtitle` | `ipc/subtitle.rs` | ✅ |
| `get_media_info` | `ipc/mediainfo.rs` | ✅ |
| `list_bookmarks` / `add_bookmark` / `delete_bookmark` | `ipc/bookmark.rs` | ✅ |
| `capture_screenshot` | `ipc/commands.rs` | ✅ |
| 崩溃插件恢复（重启用重置 error_count） | `ipc/commands.rs` | ✅ |

### 前端 UI
| 功能 | 文件 | 状态 |
|------|------|------|
| PluginManager 面板（列表/排序/开关/状态） | `PluginManager.vue` | ✅ |
| PluginPopup 通用弹窗容器 | `PluginPopup.vue` | ✅ |
| SubtitleSearch 字幕搜索组件 | `SubtitleSearch.vue` | ✅ |
| 　　・搜索输入框（自动搜索 + 手动输入） | `SubtitleSearch.vue` | ✅ |
| 　　・结果列表（文件名/语言/格式/来源） | `SubtitleSearch.vue` | ✅ |
| 　　・下载按钮 | `SubtitleSearch.vue` | ✅ |
| 　　・加载中 / 空结果 / 错误重试 状态 | `SubtitleSearch.vue` | ✅ |
| 控制栏动态插件按钮（从 manifest.ui.button 渲染） | `ControlBar.vue` | ✅ |
| 前端插件事件监听（state_changed/error/installed） | `App.vue` | ✅ |
| 错误 Toast 通知 | `App.vue` | ✅ |
| 下载成功 Toast 通知 | `App.vue` | ✅ |

---

## 尚未实现 (后续阶段)

### Phase 2 — 外置插件系统
| 功能 | 说明 | 优先级 |
|------|------|--------|
| 动态库加载（libloading） | 从 `.dylib` / `.so` / `.dll` 加载外部插件 | 高 |
| 外置插件完整生命周期 | load → enable → event → disable → unload | 高 |
| 插件热加载 / 热更新 | 文件变更时自动重载 | 中 |
| 插件市场 / 安装 UI | 从远程源浏览和安装插件 | 低 |
| 插件签名和验证 | 校验动态库完整性 | 低 |

### Phase 2+ — 功能增强
| 功能 | 说明 | 优先级 |
|------|------|--------|
| 在线字幕搜索 API | 对接 OpenSubtitles 等在线字幕源 | 高 |
| 插件超时处理（PluginError::Timeout） | 防止插件事件处理阻塞主流程 | 中 |
| 更细粒度的权限模型 | 路径白名单、网络地址过滤 | 中 |
| 插件间通信 | 一个插件可以向其他插件发事件 | 低 |
| 插件配置面板 | 每个插件可以有自己的设置页 | 低 |
| 插件开发 SDK / CLI | 脚手架生成插件项目 | 低 |

### 更多内置插件（待定）
| 插件 | 说明 |
|------|------|
| 音频均衡器 (Equalizer) | 调节低中高频 |
| 播放速度控制 (Playback Speed) | 0.5x~2x |
| 视频信息显示 (Media Info) | 编码/码率/容器详情 |
| 章节导航 (Chapters) | 基于章节标记跳转 |
| ~~书签 (Bookmarks)~~ | ~~标记时间点并快速跳转~~ | ✅ |
| 弹幕 (Danmaku) | 实时评论叠加层 |

---

## 架构要点

```
Frontend (Vue 3)         Backend (Rust/Tauri)         Plugin System
    │                          │                           │
    │── invoke ───────────────>│                           │
    │                          │── PluginBus::emit() ─────>│── Plugin A
    │                          │                           │── Plugin B
    │<── event (tauri) ────────│                           │
    │                          │── PluginRegistry ─────────│── PluginInfo[]
    │                          │                           │
    │── PluginPopup ───────────│── plugin:state_changed ──>│── UI 自动刷新
    │── SubtitleSearch ────────│── search_subtitles ──────>│── 字幕搜索
```

- 内置插件和 Tauri 命令在同一个进程，Tauri 命令直接完成实际操作
- 插件本身作为事件监听器做跟踪/统计
- Phase 2 引入动态库后，插件进程隔离会更加严格
