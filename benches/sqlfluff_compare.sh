#!/usr/bin/env bash
# reproduce the README's "60-180x faster than sqlfluff" claim. clones a real
# public dbt project, runs both linters against the rendered SQL, and prints
# a side-by-side wall-time comparison. takes a couple of minutes the first
# time because of the python venv + dbt compile.
#
# usage: bash benches/sqlfluff_compare.sh [REPO_URL]
# default REPO_URL is dbt-labs/jaffle_shop, ~25 SQL files. for the README's
# 4200-file claim, point this at a larger dbt project (a fork of jaffle_shop
# inflated by macro expansion or a real prod dbt repo). the script does not
# claim a specific multiple; it just prints the two times honestly.

set -euo pipefail

repo_url="${1:-https://github.com/dbt-labs/jaffle_shop.git}"
work=$(mktemp -d /tmp/drift-bench.XXXXXX)
trap 'rm -rf "$work"' EXIT

echo "drift bench"
echo "  repo:      $repo_url"
echo "  workdir:   $work"
echo

if ! command -v drift >/dev/null; then
    echo "drift is not on \$PATH. run \`cargo build --release\` and add target/release to PATH." >&2
    exit 2
fi

git clone --depth 1 "$repo_url" "$work/repo" >/dev/null 2>&1
cd "$work/repo"

# render the dbt project to plain SQL so both linters see the same input.
python3 -m venv "$work/venv" >/dev/null
. "$work/venv/bin/activate"
pip install --quiet 'dbt-core==1.8.*' 'dbt-postgres==1.8.*' 'sqlfluff>=3.0' >/dev/null

if [ ! -f profiles.yml ]; then
cat > "$work/profiles.yml" <<'YML'
jaffle_shop:
  target: dev
  outputs:
    dev:
      type: postgres
      host: localhost
      user: bench
      password: bench
      port: 5432
      dbname: bench
      schema: bench
      threads: 1
YML
fi
DBT_PROFILES_DIR="$work" dbt deps  >/dev/null 2>&1 || true
DBT_PROFILES_DIR="$work" dbt parse >/dev/null 2>&1 || true

target_dir="target/compiled"
if [ ! -d "$target_dir" ]; then
    # fallback: lint the raw `models/` directory if dbt could not compile (e.g. no postgres).
    target_dir="models"
fi

count=$(find "$target_dir" -name '*.sql' | wc -l | tr -d ' ')
echo "lint corpus: $count files in $target_dir"
echo

# warm fs cache
find "$target_dir" -name '*.sql' -exec cat {} + > /dev/null

bench() {
    local label="$1"; shift
    local t0 t1
    t0=$(python3 -c 'import time; print(time.time())')
    "$@" >/dev/null 2>&1 || true
    t1=$(python3 -c 'import time; print(time.time())')
    python3 -c "print(f'{(${t1} - ${t0}):.3f}')" | xargs -I {} printf "  %-12s %ss\n" "$label" {}
}

echo "drift check    $target_dir/**/*.sql"
bench "drift" drift check --dialect postgres "$target_dir"
echo
echo "sqlfluff lint  $target_dir --dialect postgres"
bench "sqlfluff" sqlfluff lint --dialect postgres "$target_dir"

echo
echo "tip: bench numbers depend hard on the corpus. for the 4200-file claim"
echo "     point this script at a real prod dbt repo or an inflated fork."
