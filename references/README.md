# references/ — igraph 三家官方实现源码（gitignored）

本目录用于存放 **igraph 三家官方实现的完整源码**，便于 AWU 推进时对照、用作 conformance fixture 提取的来源。

> **整个目录（除本 README）已加入 `.gitignore`**，不会进入 commit。每个开发者本地按下方指引克隆。

## 目录结构

```
references/
├── README.md            ← 本文件（in-repo）
├── igraph/              ← C 核心        https://github.com/igraph/igraph
├── python-igraph/       ← Python 绑定   https://github.com/igraph/python-igraph
└── rigraph/             ← R 绑定        https://github.com/igraph/rigraph
```

## 克隆

在仓库根执行：

```bash
cd references

# === igraph (C 核心) ===
# 选项 A：本地已克隆，使用符号链接（节省磁盘 + 不重复下载）
ln -s /Users/dovchen/code/igraph igraph
# 锁定到稳定版（如未完成 submodule，下行不可省）
( cd igraph && git checkout v1.0.0 && git submodule update --init --recursive )

# 选项 B：全新克隆
# git clone --depth 1 --branch v1.0.0 https://github.com/igraph/igraph.git
# ( cd igraph && git submodule update --init --recursive )

# === python-igraph ===
git clone --depth 1 https://github.com/igraph/python-igraph.git
# 锁定到与 oracle.py 中 pip 安装版本一致
( cd python-igraph && git checkout 0.11.x )

# === R-igraph (注意 repo 名是 rigraph) ===
git clone --depth 1 https://github.com/igraph/rigraph.git

cd ..
```

## 验证

```bash
ls references/igraph/src/linalg/arpack.c            # ARPACK 翻译参照
ls references/igraph/src/isomorphism/bliss/graph.cc # BLISS 翻译参照
ls references/python-igraph/tests/                  # python-igraph 测试集（526 方法）
ls references/rigraph/tests/testthat/               # R-igraph testthat 测试集
```

## 锁定版本

每次克隆/升级请同步更新 `.codefuse/tracking/REFERENCES.md` 中的 commit hash 与日期，保证可复现。

## 与脚本的关系

| 脚本 | 读取的 references 路径 |
|------|-----------------------|
| `scripts/oracle.py` | 不直接读；运行通过 pip 安装的 python-igraph |
| `scripts/test_extract/from_c.py` | `references/igraph/tests/unit/*.c` + `*.out` |
| `scripts/test_extract/from_py.py` | `references/python-igraph/tests/test_*.py` |
| `scripts/test_extract/run_r.R` | `references/rigraph/tests/testthat/test-*.R` |
| AWU Step 1 (Recon) | 按需读 `references/igraph/src/.../*.c` |

## 升级

升级时同步：
1. 切到新 tag/branch（`git -C references/<repo> fetch && git -C references/<repo> checkout <tag>`）
2. 更新 `.codefuse/tracking/REFERENCES.md`
3. 重跑 `scripts/test_extract/*` 重新提取 conformance fixture
4. 更新 `oracle.py` 依赖的 pip 版本（如 python-igraph）
5. CI 全量回归一次
