# 项目结构

```
rust-igraph/
├── Cargo.toml                         工作区根
├── src/                               主库源码
│   ├── core/                          数据结构 (Graph, Vector, Matrix…)
│   └── algorithms/                    算法实现
├── crates/igraph-wasm/                WASM 绑定
├── tests/                             集成测试
├── examples/                          115 个可运行示例
├── book/                              mdBook 文档（你正在阅读的）
├── website/                           官网 + Playground
├── docs/plans/                        规划文档
├── .codefuse/tracking/                算法追踪、架构决策
├── scripts/                           Oracle 脚本 + 一致性提取器
├── fixtures/                          标准图数据集 (karate, dolphins…)
└── tests/conformance/{c,py,r}/        上游测试套件提取
```

> 详细说明请参阅 [English Project Layout](/rust-igraph/book/project-layout.html)。
