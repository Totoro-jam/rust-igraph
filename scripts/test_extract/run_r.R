# Placeholder. Phase-1 fully automated extractor.
#
# When R and the rigraph package are installed, this script walks every
# `tests/testthat/test-*.R` file in `references/rigraph/`, captures every
# `expect_equal(actual, expected)` pair as an R object, and writes JSON
# fixtures matching the wire format documented in `__init__.py`.
#
# Usage (Phase 1+):
#   Rscript scripts/test_extract/run_r.R --algo bfs
#
# Phase 0 ships only the hand-curated manifest in `from_r.py`; this file is
# kept so the extraction-flow boundary is explicit.

cat("run_r.R: not yet implemented (Phase 1 work).\n",
    "For Phase 0 use scripts/test_extract/from_r.py instead.\n",
    sep = "")
quit(status = 1)
