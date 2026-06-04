# AI 工作流

rust-igraph 使用 AI 辅助开发，通过 Claude Code 协作完成大规模移植工作。

## 工具链

- **Claude Code** — 主要的 AI 编码助手
- **AWU Skills** — 自定义技能 (`/awu-start`, `/awu-translate`, `/awu-test` 等)
- **Sub-agents** — 专用子代理（翻译、测试、文档等）

## 质量保证

AI 生成的代码必须通过：

1. `cargo clippy` + `cargo fmt` — 代码质量
2. 单元测试 + proptest — 正确性
3. 三源一致性 — 数值精度
4. Code review — 人工审查

详情参阅 [AI Workflow (English)](/rust-igraph/book/ai-workflow.html)
