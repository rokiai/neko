# Neko 开发约束

- 保持当前 Tauri 迁移方向，不重新引入 Electron 运行时或已删除的视频能力。
- Rust 使用 `src-tauri/rust-toolchain.toml` 固定的 Rust `1.95.0`，不要为旧环境降低 edition 或依赖版本。
- 修改后按影响范围运行必要的格式检查、lint、类型检查、测试和构建；不要提交 `target`、`dist`、`out`、`.pnpm-store` 等生成物、密钥或用户配置。
- 优先修复根因，保持改动聚焦，不做无关重构或无调用方的抽象。
- 每次完成代码修改并通过必要验证后，必须使用 `.agents/skills/adversarial-review/SKILL.md` 做对抗式复核；复核阶段只读，不直接修改代码。
- 使用 `apply_patch` 修改文件；除非用户明确要求，不提交 commit、不创建分支。
