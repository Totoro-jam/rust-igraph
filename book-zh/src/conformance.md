# 一致性矩阵

rust-igraph 通过对比三个官方实现来验证数值正确性：

| 来源 | 提取方式 | 测试数 |
|------|----------|--------|
| igraph C | 解析 `*.c` + `*.out` 测试 | 425+ |
| python-igraph | 解析 pytest 方法 | 526+ |
| R-igraph | 解析 testthat 用例 | 108+ |

查看完整矩阵：
[CONFORMANCE.md on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/.codefuse/tracking/CONFORMANCE.md)
