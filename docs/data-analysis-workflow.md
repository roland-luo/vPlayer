# vPlayer 数据分析流程

> 适用范围：vPlayer 桌面端播放器（Tauri + Rust + libmpv）的用户行为、性能指标与错误日志分析。

---

## 1. 需求确认

在开始任何分析前，明确以下问题：

| 问题 | 说明 |
|------|------|
| 分析目标 | 用户留存？播放体验？错误归因？功能使用频率？ |
| 时间范围 | 具体日期区间，是否包含版本发布节点 |
| 样本范围 | 全量用户 / 特定平台 / 特定版本 |
| 输出形式 | 可视化报表 / 原始数据导出 / 结论摘要 |

---

## 2. 数据源梳理

vPlayer 当前可采集的数据包括：

| 数据类型 | 来源 | 关键字段 |
|----------|------|----------|
| 播放行为 | 前端事件埋点 | `play`, `pause`, `seek`, `fullscreen`, `volume_change` |
| 性能指标 | Rust 后端日志 | 首帧耗时、缓冲次数、CPU/GPU 占用、内存峰值 |
| 错误日志 | Tauri 主进程 + libmpv | `libmpv` 初始化失败、`OpenGL` 上下文异常、插件崩溃、纹理分配失败 |
| 环境信息 | 启动时上报 | OS 版本、GPU 型号、驱动版本、libmpv 版本、屏幕分辨率 |

> **注意**：所有数据采集须遵守隐私政策，敏感路径信息需脱敏处理。

---

## 3. 数据清洗与预处理

```
原始日志
    │
    ▼
┌─────────────────┐
│ 1. 格式标准化   │  → 统一时间戳、字段命名、枚举值
│ 2. 去重与过滤   │  → 剔除测试设备、内部调试日志
│ 3. 关联补齐     │  → 将前端事件与后端性能日志按 session_id 关联
│ 4. 异常标记     │  → 标记 GPU 驱动过旧、虚拟机环境等已知异常模式
└─────────────────┘
    │
    ▼
清洗后数据集
```

---

## 4. 分析执行

### 4.1 播放体验分析

- **首帧时间分布**：按平台 / GPU 型号分桶，识别 P50/P95/P99
- **缓冲频率**：统计 `buffering` 事件密度，定位网络或解码瓶颈
- **播放完成率**：`play` → `ended` 的漏斗转化，识别中途退出高峰时段

### 4.2 错误归因分析

- **错误聚类**：按错误类型（OpenGL / libmpv / 插件 / 纹理分配）聚合
- **环境关联**：交叉分析错误率与 GPU 驱动版本、操作系统版本
- **影响面评估**：计算受影响用户占比、对播放完成率的影响

### 4.3 功能使用分析

- **功能渗透率**：倍速播放、字幕切换、画面比例调整、插件启用率
- **高频操作路径**：常见的热键组合或菜单点击序列
- **插件健康度**：各插件的调用次数、错误率、平均响应耗时

---

## 5. 结果输出

| 输出物 | 内容 | 责任人 |
|--------|------|--------|
| 数据看板 | Grafana / 自建面板，核心指标实时可视化 | 数据工程师 |
| 分析报告 | Markdown / PDF，含结论、图表、建议 | 分析师 |
| 问题单 | 对高优问题创建 GitHub Issue，关联相关日志样本 | 分析师 + 开发 |

---

## 6. 复盘与迭代

1. **指标有效性 review**：当前指标是否真实反映用户体验？
2. **埋点覆盖检查**：新功能上线后是否及时补充埋点？
3. **告警阈值调优**：错误率 / 性能指标的告警线是否合理，避免噪音？

---

## 附录：常用查询示例

### 按 GPU 型号统计 libmpv 初始化失败率

```sql
SELECT
    gpu_model,
    COUNT(*) AS total_sessions,
    SUM(CASE WHEN error_type = 'libmpv_init_failed' THEN 1 ELSE 0 END) AS fail_count,
    ROUND(fail_count * 100.0 / total_sessions, 2) AS fail_rate_pct
FROM session_logs
WHERE date >= '2026-04-01'
GROUP BY gpu_model
HAVING total_sessions >= 100
ORDER BY fail_rate_pct DESC;
```

### 插件崩溃隔离效果评估

```sql
SELECT
    plugin_name,
    COUNT(*) AS crash_count,
    AVG(session_duration_after_crash_sec) AS avg_remaining_session
FROM plugin_crash_logs
WHERE date >= '2026-04-01'
GROUP BY plugin_name;
```
