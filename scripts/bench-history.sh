#!/usr/bin/env bash
set -euo pipefail

# Run the Cyrius benches for the series Rust benchmarked, append one row per bench to the history CSV,
# and regenerate the port's table in docs/benchmarks.md (baseline -> previous -> current, beside Rust's
# record).
#
# Usage:
#   ./scripts/bench-history.sh                      # appends to docs/benchmarks/history.csv
#   ./scripts/bench-history.sh other-history.csv    # a different history file, relative to where you run it;
#                                                   # its machine rows go to runs.csv beside it
#
# ⭐ WHAT IT READS. Each bench prints `@row <series> <picoseconds per op>` after its bench_report line, and
# that is the only thing parsed here, so the human line's format can change without breaking the history.
# The series names are Rust's (`renderer/render_frame/1`, `screen_capture/png_encode/full_screen/4K`, ...),
# so a row lines up with its Rust row in docs/benchmarks/rust-a460c3a.csv.
#
# ⛔ A FAILED BENCH RECORDS NOTHING. Every bench checks that its op did the work (a frame that drew the
# surfaces, a band of the right extent, a capture that was not refused) and exits non-zero otherwise; one
# failure aborts the whole run before the CSV is touched, so the history never holds a timed early return.
#
# ⚠ THE MACHINE IS RECORDED PER RUN (docs/benchmarks/runs.csv: CPU model, clocksource, toolchain), because
# the numbers mean nothing without it: Rust's record carries no machine at all, and the bench timer floor
# moves with the clocksource (see the header of lib/bench.cyr).
#
# No network. Needs bash, awk, git (for the commit, optional) and the cyrius toolchain.

ROOT=$(cd "$(dirname "$0")/.." && pwd)
CALLER_PWD=$(pwd)
cd "$ROOT"

# A relative argument means relative to where the script was RUN, not to the repo root it cds into.
HISTORY_FILE="docs/benchmarks/history.csv"
if [ $# -ge 1 ]; then
    case "$1" in
        /*) HISTORY_FILE="$1" ;;
        *)  HISTORY_FILE="$CALLER_PWD/$1" ;;
    esac
fi
# The machine record lives beside the history it describes (the default: docs/benchmarks/runs.csv).
RUNS_FILE="$(dirname "$HISTORY_FILE")/runs.csv"
RUST_FILE="docs/benchmarks/rust-a460c3a.csv"
BENCHMARKS_MD="docs/benchmarks.md"
BENCHES=(tests/compositor_bench.bcyr tests/screen_capture_bench.bcyr)

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
# A run over uncommitted benches or source is not the commit it names; say so in the commit column.
if [ "$COMMIT" != "unknown" ] && [ -n "$(git status --porcelain -- src tests lib 2>/dev/null)" ]; then
    COMMIT="${COMMIT}+dirty"
fi
BRANCH=$(git branch --show-current 2>/dev/null || true)
if [ -z "$BRANCH" ]; then BRANCH="unknown"; fi
CPU=$(awk -F': ' '/^model name/ { print $2; exit }' /proc/cpuinfo 2>/dev/null || true)
if [ -z "$CPU" ]; then CPU="unknown"; fi
CPU=$(printf '%s' "$CPU" | tr ',' ';')
CLOCK=$(cat /sys/devices/system/clocksource/clocksource0/current_clocksource 2>/dev/null || echo "unknown")
PIN=$(awk -F'"' '/^cyrius[[:space:]]*=/ { print $2; exit }' cyrius.cyml)
TOOL=$(cyrius --version 2>/dev/null | awk 'NR == 1 { print $2 }')
if [ -z "$TOOL" ]; then TOOL="unknown"; fi

if ! grep -q '^<!-- port-table:begin' "$BENCHMARKS_MD" || ! grep -q '^<!-- port-table:end' "$BENCHMARKS_MD"; then
    echo "bench-history: ${BENCHMARKS_MD} has no port-table markers; refusing to rewrite it" >&2
    exit 1
fi

echo "aethersafha benchmarks"
echo "  commit: ${COMMIT}   branch: ${BRANCH}   date: ${TIMESTAMP}"
echo "  machine: ${CPU} (clocksource ${CLOCK})   cyrius ${TOOL} (pin ${PIN})"
echo ""

ROWS=$(mktemp)
TABLE=$(mktemp)
NEWMD=$(mktemp)
trap 'rm -f "$ROWS" "$TABLE" "$NEWMD"' EXIT

for bench in "${BENCHES[@]}"; do
    echo "== ${bench}"
    if ! out=$(cyrius bench "$bench" 2>&1); then
        printf '%s\n' "$out" | grep -v '^warning' || true
        echo "bench-history: ${bench} FAILED; nothing recorded" >&2
        exit 1
    fi
    printf '%s\n' "$out" | grep -E '^=== |^  [A-Za-z_]+/[^ ]+: |^  \[' || true
    printf '%s\n' "$out" | awk '$1 == "@row" { printf "%s,%.3f\n", $2, $3 / 1000 }' >> "$ROWS"
done

COUNT=$(wc -l < "$ROWS" | tr -d ' ')
if [ "$COUNT" -eq 0 ]; then
    echo "bench-history: the benches printed no @row lines; nothing recorded" >&2
    exit 1
fi

mkdir -p "$(dirname "$HISTORY_FILE")"
if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,commit,branch,benchmark,estimate_ns" > "$HISTORY_FILE"
fi
awk -v ts="$TIMESTAMP" -v c="$COMMIT" -v br="$BRANCH" '{ print ts "," c "," br "," $0 }' "$ROWS" >> "$HISTORY_FILE"

if [ ! -f "$RUNS_FILE" ]; then
    echo "timestamp,commit,branch,cpu,clocksource,cyrius" > "$RUNS_FILE"
fi
echo "${TIMESTAMP},${COMMIT},${BRANCH},${CPU},${CLOCK},${TOOL}" >> "$RUNS_FILE"

# The table: every series of the latest port run (in its order), Rust's two runs beside it, and up to
# three port runs — the first ever (baseline), the one before this (previous) and this one (current).
# Deltas are against the PORT's baseline only. Rust's column is a record from an unknown machine; no
# delta is computed across the two.
awk -F',' -v rustf="$RUST_FILE" -v runsf="$RUNS_FILE" '
function fmt_ns(ns) {
    if (ns >= 1000000000) { return sprintf("%.2f s", ns / 1000000000) }
    if (ns >= 1000000)    { return sprintf("%.1f ms", ns / 1000000) }
    if (ns >= 1000)       { return sprintf("%.1f µs", ns / 1000) }
    if (ns >= 100)        { return sprintf("%.1f ns", ns) }
    return sprintf("%.2f ns", ns)
}
function delta(old, new,    pct) {
    if (old == 0) { return "" }
    pct = (new - old) / old * 100
    if (pct < -3) { return sprintf(" **%+.0f%%**", pct) }
    if (pct > 3)  { return sprintf(" %+.0f%%", pct) }
    return ""
}
BEGIN {
    while ((getline line < rustf) > 0) {
        split(line, f, ",")
        if (f[1] == "timestamp") { continue }
        if (!(f[1] in rseen)) { rseen[f[1]] = 1; rts[++nr] = f[1] }
        rval[f[1], f[4]] = f[5]
    }
    while ((getline line < runsf) > 0) {
        split(line, f, ",")
        if (f[1] == "timestamp") { continue }
        rcommit[f[1]] = f[2]; rcpu[f[1]] = f[4]; rclock[f[1]] = f[5]; rtool[f[1]] = f[6]
    }
}
FNR == 1 && $1 == "timestamp" { next }
{
    if (!($1 in seen)) { seen[$1] = 1; ts[++n] = $1 }
    if (!(($1, $4) in val)) { cnt[$1]++; ord[$1, cnt[$1]] = $4 }
    val[$1, $4] = $5
    commit[$1] = $2
}
END {
    if (n == 0) { exit 1 }
    cur = ts[n]
    np = 0
    if (n >= 3) { pick[++np] = ts[1]; label[np] = "Port baseline"; pick[++np] = ts[n - 1]; label[np] = "Port previous" }
    if (n == 2) { pick[++np] = ts[1]; label[np] = "Port baseline" }
    pick[++np] = cur; label[np] = "Port current"

    print "Port runs in this table (UTC, from `docs/benchmarks/runs.csv`):"
    print ""
    print "| Column | Run | Commit | Machine | Clocksource | cyrius |"
    print "|---|---|---|---|---|---|"
    for (i = 1; i <= np; i++) {
        t = pick[i]
        m = (t in rcpu) ? rcpu[t] : "unknown"
        k = (t in rclock) ? rclock[t] : "unknown"
        v = (t in rtool) ? rtool[t] : "unknown"
        printf "| %s | %s | `%s` | %s | %s | %s |\n", label[i], t, commit[t], m, k, v
    }
    print ""

    hdr = "| Series"
    sep = "|---"
    for (j = 1; j <= nr; j++) { hdr = hdr sprintf(" | Rust `a460c3a` run %d", j); sep = sep "|---:" }
    for (i = 1; i <= np; i++) { hdr = hdr " | " label[i]; sep = sep "|---:" }
    print hdr " |"
    print sep "|"
    # Series order: the current run first, then any series only an older picked run has.
    ns = 0
    for (k = 1; k <= cnt[cur]; k++) { b = ord[cur, k]; if (!(b in listed)) { listed[b] = 1; order[++ns] = b } }
    for (i = 1; i < np; i++) {
        t = pick[i]
        for (k = 1; k <= cnt[t]; k++) { b = ord[t, k]; if (!(b in listed)) { listed[b] = 1; order[++ns] = b } }
    }
    for (s = 1; s <= ns; s++) {
        b = order[s]
        row = "| `" b "`"
        for (j = 1; j <= nr; j++) {
            row = row " | " (((rts[j], b) in rval) ? fmt_ns(rval[rts[j], b] + 0) : "—")
        }
        for (i = 1; i <= np; i++) {
            if (!((pick[i], b) in val)) { row = row " | —"; continue }
            cell = fmt_ns(val[pick[i], b] + 0)
            if (i > 1 && ((pick[1], b) in val)) { cell = cell delta(val[pick[1], b] + 0, val[pick[i], b] + 0) }
            row = row " | " cell
        }
        print row " |"
    }
}' "$HISTORY_FILE" > "$TABLE"

awk -v table="$TABLE" '
/^<!-- port-table:begin/ { print; while ((getline line < table) > 0) { print line }; skip = 1; next }
/^<!-- port-table:end/   { skip = 0 }
!skip { print }
' "$BENCHMARKS_MD" > "$NEWMD"
cat "$NEWMD" > "$BENCHMARKS_MD"

echo ""
echo "  ${COUNT} benchmarks recorded"
echo "  CSV:      ${HISTORY_FILE}"
echo "  Runs:     ${RUNS_FILE}"
echo "  Markdown: ${BENCHMARKS_MD}"
