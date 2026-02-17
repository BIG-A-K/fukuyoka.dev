#!/usr/bin/env bash
set -euo pipefail

DB="${1:-akasha}"
K="${2:-10}"
PGUSER="${POSTGRES_USER:-akasha}"

# 0埋めの768次元ベクトル文字列を作る: [0,0,0,...]
zeros=""
for ((i=0; i<768; i++)); do
  zeros+="0,"
done
zeros="${zeros%,}"
QVEC="[${zeros}]"

psql -U "${PGUSER}" -d "${DB}" -v ON_ERROR_STOP=1 <<SQL
\timing on

-- 1) 計画と実測（インデックス使用の確認）
SET enable_seqscan = off;

EXPLAIN (ANALYZE, BUFFERS, VERBOSE)
SELECT id, title
FROM ${POSTGRES_TABLE:-documents}
ORDER BY embeds <=> ('$QVEC'::vector(768))
LIMIT $K;
SQL
