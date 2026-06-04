# 项目结构

```
rust-igraph/
├── Cargo.toml                         工作区根
├── CLAUDE.md                          AI 代理的项目级规则
├── CONTRIBUTING.md                    Alpha 阶段外部贡献策略
├── DEVELOPMENT.md                     设置 + AWU 工作流（维护者笔记）
├── CHANGELOG.md                       变更日志
├── SECURITY.md                        安全漏洞报告
├── LICENSE                            GPL-2.0-or-later
├── README.md                          项目概览
├── deny.toml                          cargo-deny: 许可证 + 安全策略
│
├── src/
│   ├── lib.rs                         crate 根 + 重导出
│   ├── core/                          Graph, Vector, Matrix, 错误类型
│   └── algorithms/                    所有算法实现
│
├── tests/
│   └── conformance/{c,py,r}/<algo>/   三源一致性数据（已跟踪）
│
├── benches/                           criterion 基准测试
├── examples/                          可运行示例
├── fixtures/                          标准图数据集 (karate, dolphins…)
├── scripts/
│   ├── oracle.py                      python-igraph 实时对比桥
│   ├── bench_compare.py               criterion vs python-igraph 差异
│   └── test_extract/                  一致性提取器 (from_c/py/r)
├── templates/                         AWU SOP 第 3 步骨架模板
│
├── book/                              英文 mdBook 站点
├── book-zh/                           中文 mdBook 站点（你正在阅读的）
├── docs/plans/                        规划与设计文档
│
├── .codefuse/tracking/                与代码一起提交的追踪文件
│   ├── ALGORITHMS.md                    AWU 状态（唯一真相源）
│   ├── ARCHITECTURE.md                  架构决策记录索引
│   ├── CONFORMANCE.md                   三源覆盖矩阵
│   ├── AI_PROMPTS.md                    有效提示词手册
│   ├── RESUME.md                        会话逐次笔记
│   └── perf/<ALGO-XXX>.json             criterion 基准快照
│
├── .claude/                           AI 基础设施（已提交）
│   ├── agents/                          7 个子代理
│   ├── skills/                          9 个 /awu-* 技能 + 辅助工具
│   └── hooks/                           git 安全钩子 + 自动格式化
├── .githooks/                         仓库级 git 钩子 (Co-Authored-By)
│
├── website/                           官网 + Playground
├── references/                        gitignored: igraph C / py / R 克隆
└── .github/workflows/                 CI + Pages 部署
```
