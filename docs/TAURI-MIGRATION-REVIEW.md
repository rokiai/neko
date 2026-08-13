# Neko Tauri 迁移评审报告与治理清单

> 评审基线：commit `fd5a5b9`（Fix macOS Break Spaces follow）+ 工作区未提交改动（`platform/windows_desktops.rs` 新增、`platform/mod.rs`、`Cargo.toml`）。
>
> 对照物：`docs/TAURI-MIGRATION.md` 全文；迁移前 Electron 实现（`1ce4c92^` 的 `src/main/lib/*`）。
>
> 评审日期：2026-08-13。方法：全量通读 Rust/前端核心代码，逐条对照迁移文档与旧实现语义，运行验证套件。
>
> **修复日期：2026-08-13（同日）。N1–N14 全部处理完毕，状态见各条目；本机无法验证的项（Windows 编译、真机锁屏）已在条目内注明。**

## 验证快照

| 检查                                        | 评审时                         | 修复后                                                                                                |
| ------------------------------------------- | ------------------------------ | ----------------------------------------------------------------------------------------------------- |
| `cargo fmt --all --check`（src-tauri 内）   | 通过                           | 通过                                                                                                  |
| `cargo clippy --all-targets -- -D warnings` | 通过                           | 通过                                                                                                  |
| `cargo test`                                | 3 个（仅 config）              | **15 个**（config 3 + schema 4 + idle/锁屏 3 + 调度转移 5）                                           |
| `pnpm typecheck` / `pnpm test`              | 通过（10 用例）                | 通过（10 用例）                                                                                       |
| Windows/macOS 编译验证                      | 无（release tag 才第一次编译） | CI 新增 `platform-check`（windows-latest / macos-latest `cargo check --all-targets`），下次 push 生效 |

## 总结论：有争议 → 已治理

架构与社区方案（Tauri 2 官方模板、官方插件、capabilities、三平台 release 矩阵）高度对齐；配置迁移、多屏 Break 窗口、command 契约质量高于社区平均。评审发现 1 个真实调度 bug（N1）、Windows 锁屏能力缺失（N2）、一组 macOS 可感知体验回归（N3–N5）及若干工程治理项，均已修复；Linux 锁屏经核实为 Electron 基线即有的缺口（非回归），已在文档中显式登记。

---

## 治理清单

状态标记：`[ ]` 待修 / `[x]` 已修并通过验收 / `[~]` 已修但需真机/CI 二次确认。

### N1 [高] 短锁屏（低于阈值）解锁后误触发"空闲重置" — [x]

- 位置（评审时）：`scheduler.rs` tick 非空闲分支 + `monitors/idle.rs` `read_status` 解锁时无条件返回上一轮 `lock_start_at_ms`。
- 失败场景：锁屏 5 秒再解锁（阈值默认 300 秒）→ 重置 `last_completed_at_ms`、清零 `postponed_count`；开启 `idleResetNotification` 时误弹 "Timer reset" 通知。
- 依据：文档 §7.2 "锁屏持续超过阈值后才触发重置"；旧 Electron `checkIdle` 仅 `lockedFor > threshold` 才算 idle。
- **修复记录**：`monitors/idle.rs` 抽出纯函数 `evaluate`，解锁后仅当锁屏时长 ≥ 阈值才返回 `lock_start_at_ms`（且只返回一次）。单测覆盖：短锁屏解锁不重置、长锁屏解锁重置且仅一次、普通 idle 需 `idleResetEnabled`。`scheduler/transitions.rs` 另有 tick 层面的短/长锁屏转移测试双保险。

### N2 [高] Windows 无锁屏检测（Linux 核实为基线即缺，非回归） — [~]

- 位置（评审时）：非 macOS `is_screen_locked` 恒 `false`。
- 失败场景：Windows 锁屏后调度器认为用户在工作：Break 照常弹出、倒计时流逝、统计照记；`pendingBreakDue` 补弹不会发生。旧 Electron `powerMonitor` 在 Windows 支持 `'locked'`，属回归。
- 更正：Linux 的 Electron `powerMonitor` 同样不报 `locked`（API 仅 macOS/Windows），Linux 部分**不是回归**，是从未存在的能力。
- **修复记录**：平台锁屏探测集中到新模块 `monitors/lock.rs`。Windows 用 `WTSQuerySessionInformationW(WTSSessionInfoEx)` 判定 `SessionFlags == WTS_SESSIONSTATE_LOCK`（轮询友好、Win10+；Win7 取反语义不支持，已注释）。API 签名逐一对照 windows crate 0.62 生成源核实。Linux 维持基线（恒 `false`）并在 §7.2 登记为已知缺口及后续 D-Bus 方案。
- 待确认：本机缺 `llvm-rc` 无法交叉编译，Windows 分支编译由 CI `platform-check`（N8）保障；锁屏→解锁→补弹链路需 Windows 真机手测。

### N3 [中] 托盘菜单/系统通知丢失国际化，丢失"关于"与禁用剩余时间 — [x]

- 位置（评审时）：托盘全部英文硬编码；通知模式 Break 与空闲重置通知英文硬编码；无 About 项；禁用状态不显示剩余时间。
- 依据：文档 §1.1 "保留 Neko 的……国际化"；§9.1 菜单语义清单。
- **修复记录**：新增 `core/i18n.rs`——与 `src/shared/i18n/{en,zh,ja}.ts` 同源的最小词表（托盘全部文案、空闲重置通知、Break 默认标题/正文），`settings.locale=system` 时经 `sys-locale` 探测系统语言。托盘补回 About 项（同"设置"打开设置窗）与 `tray.disabledLeft`（"已关闭 · 剩余 {time}"）。通知模式 Break 的标题/正文兜底与空闲重置通知全部走词表。
- 验收方式：切换 locale 后重建菜单即生效（菜单签名包含 locale）；文案与前端词条一致（zh 的托盘词条按前端 `zh.ts` 逐条对照）。

### N4 [中] macOS Dock 点击无法恢复设置窗口（Reopen 未处理） — [x]

- 位置（评审时）：`lib.rs` `.run(ctx)` 不处理运行事件。
- **修复记录**：`lib.rs` 改为 `.build(ctx)` + `app.run(closure)`，macOS 下 `RunEvent::Reopen` 调 `platform::show_settings`（已确认 tauri 2.11.5 中该变体为 macOS-only cfg，match 臂带 `#[cfg]`）。
- 待手测：最小化设置 → 点 Dock 图标恢复；隐藏设置（Accessory）与退出行为不变。

### N5 [中] macOS 菜单栏图标不是 template image — [x]

- **修复记录**：`scripts/make-tray-template.py` 生成 Neko 猫形 template 图标（36×36 = 18pt@2x，黑 glyph + alpha）至 `src-tauri/icons/tray-template.png`，macOS 托盘经 `Image::from_bytes`（tauri `image-png` feature）加载并 `icon_as_template(true)`；Windows/Linux 继续用彩色应用图标。
- 待手测：深/浅色菜单栏观感。

### N6 [中] 每秒全量重建托盘菜单 — [x]

- 位置（评审时）：tick 每秒 `refresh_tray` → 全量新建 ~15 个 MenuItem 并 `set_menu`。
- **修复记录**：托盘拆到 `platform/tray.rs` 并改造刷新策略——tooltip 与 macOS 标题保持每秒更新（秒级精度）；菜单内容做签名（locale + 启停状态 + 状态行文案），签名变化才重建 `set_menu`。菜单内状态行的倒计时改用分钟精度（"27m 后休息"），即菜单至多每分钟重建一次、状态切换即时重建。旧 Electron 菜单为 5 秒粗刷、展开时同样有 5 秒级过期，故菜单分钟精度不劣于基线。

### N7 [中] Settings 为无类型 `serde_json::Value`，违反文档 DTO 契约 — [x]

- **修复记录**：采纳方案 a。新增 `config/schema.rs`：`SettingsSchema` 与 `src/shared/settings.ts` 逐字段一致（含枚举值），在 `settings_set` / `break_preview` 边界做 `serde_path_to_error` 校验，类型错误返回指明字段路径的可展示错误（如 `` `breakFrequencySeconds`: invalid type: string ``）；加载旧配置校验失败仅告警不拒载（按键回退仍兜底）。运行时读取保持 `Value` + fallback，渐进迁移。文档 §6.1 已补"实际落地方式"说明。
- 单测：默认配置通过校验、错误类型带字段名拒绝、非法枚举拒绝、normalize 后的部分输入合法。

### N8 [中] 平台特定代码缺少提交前编译验证 — [~]

- **修复记录**：`ci.yml` 新增 `platform-check` job（windows-latest + macos-latest 矩阵，固定 Rust 1.95.0，`cargo check --all-targets`，含 rust-cache，不 bundle）。
- 待确认：下次 push 观察该 job 通过；可选做一次故意破坏验证拦截能力。

### N9 [中] scheduler.rs / platform/mod.rs 超模块化硬限，调度状态机零测试 — [x]

- **修复记录**：全部拆分并低于 350 行硬限：
  - `scheduler.rs` 201（API/状态查询）、`scheduler/breaks.rs` 258（休息生命周期）、`scheduler/tick.rs` 170（tick + 触发 + 通知）、`scheduler/transitions.rs` 275（**纯状态转移 + 5 个单测**：短锁屏保持、长锁屏单次重置、idle 期间到期补弹一次、睡眠间隙重排、工作秒数冲刷）、`scheduler/util.rs` 84、`state.rs` 206。
  - `platform/mod.rs` 342（窗口）、`platform/tray.rs` 337、`platform/dock.rs` 24。
  - 顺带 `config.rs` 290 + `config/persist.rs`（文件 IO/旧路径）+ `config/schema.rs`（原 391 行超限）。
- `transitions::apply` 不依赖 `AppHandle`/时钟/OS 探测，逐行保序自原 tick，行为无意变化。

### N10 [低] 无持久化日志，迁移日志在 release 包无处可查 — [x]

- **修复记录**：改用官方 `tauri-plugin-log`（社区标准做法）：Stdout + LogDir（`neko.log`，app_log_dir），级别 Info、tao/wry 降为 Warn。`tracing` 开启 `log` feature 桥接（未安装 tracing subscriber 时事件转发至 log facade），既有 `tracing::warn!` 等调用零改动。删除 `utils.rs` 与 `tracing-subscriber` 依赖。setup 增加一条启动 info 日志（含版本）作为日志锚点。
- 已验证：dev 冒烟运行确认 `~/Library/Logs/com.neko.app/neko.log` 创建且桥接事件落盘（`[neko_lib][INFO] Neko starting version="0.1.3"`）。

### N11 [低] `runtime_status` 不滚动 dayKey，跨天后 Today 面板显示昨日数据 — [x]

- **修复记录**：`state.rs` 新增只读 `today_stats_snapshot`（dayKey 过期即返回零值新日，不写盘），`runtime_status` 改用之；写路径仍由 `ensure_today_stats`/`record_break` 滚动。

### N12 [低] capabilities 过度授权 — [x]

- **修复记录**：前端实际仅用 `invoke` + `listen`，`capabilities/default.json` 收敛为 `["core:default"]`（core:default 已含 event listen/emit 默认权限）。顺带移除零调用方的 `tauri-plugin-process`（退出走 `app.exit`）。tauri-build 的权限校验在编译期证实无缺权限。

### N13 [低] 迁移文档与实现脱节 — [x]

- **修复记录**：`TAURI-MIGRATION.md` 已修订：阶段 A/B 全勾、C/D 按实情勾选并注明真机验收缺口；§16.2 Dock 策略改为"跟随设置窗口可见性"（与实现一致并说明原因）；§5.2 标注 `neko://sound/*`（rodio 直播）与 `neko://runtime/status`（前端轮询）的实际取舍；§3.2 标注实际启用插件集与 capabilities 现状；§7.2 更新三平台锁屏实现与 Linux 已知缺口；§15 当前状态重写。

### N14 [低] rust 脚本从仓库根运行时不读取 `rust-toolchain.toml` — [x]

- **修复记录**：`package.json` 的 `rust:format:check` / `rust:lint` / `rust:test` / `tauri:check` 全部改为 `cd src-tauri && cargo ...`，rustup 按 cwd 正确解析 1.95.0。本机（默认 stable 1.78）验证 `pnpm rust:test` 直接可用。

---

## 遗留事项（修复后仍开放）

| 事项                   | 来源         | 说明                                                                                                                                                |
| ---------------------- | ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows 真机验收       | N2/N8/§16.3  | WTS 锁屏链路、虚拟桌面跟随、多屏 Break；CI check 只保证编译                                                                                         |
| Linux 真机验收         | §16.3        | X11/Wayland 透明遮罩、appindicator 托盘、菜单签名重建行为                                                                                           |
| Linux 锁屏检测         | N2           | 基线即缺；如需补齐走 D-Bus（freedesktop/GNOME/login1）三探测，属新能力而非修复                                                                      |
| CI platform-check 首跑 | N8           | 下次 push 确认矩阵通过                                                                                                                              |
| macOS 手测一轮         | N3–N6/N12    | 托盘三语菜单与 template 图标观感、Dock 点击恢复、Break 全流程（capabilities 收敛后 event listen 走 core:default，dev 冒烟启动正常，弹出流程待点验） |
| Notification 模式手测  | 文档阶段 D   | 无 Break 窗、声音、统计一次性验收                                                                                                                   |
| updater 接入           | 文档阶段 D/E | 未配置 endpoint，生产自动更新保持关闭                                                                                                               |

## 已确认无需处理（勿当作问题反复触碰）

- tick 中"锁屏超过 frequency"分支与旧版行为不同（新版设置 `idle_start` 并清 `break_time`），更符合文档 §7.2 语义，判定为合理改进。
- `complete_break_tracking` 先清 `break_started_at`、`end_popup_break` 再判空的双重记账防护，设计正确。
- 配置迁移顺序（只读探测 → 原子写成功 → 才备份 `.bak` → `migration_version` 防重复）完全符合 §6.2，`ReplaceFileW` 原子替换保留（现于 `config/persist.rs`）。
- Break 快照/推迟 UI 无 snooze 按钮：迁移前即如此，非回归。
- macOS NSPanel anchor 与 Windows `IVirtualDesktopManager` 方案：比社区常见做法更稳，保留（注释已说明权衡）。
- preview 语义（临时 settings、不写盘、恢复原调度与推迟计数）与旧版一致。
- 托盘状态行顺序（禁用 → 休息中 → 非工作时间 → 空闲 → 倒计时）与旧版（禁用 → 非工作时间 → 空闲 → 休息中）略有差异："休息中"优先展示更合理，保留新序。
