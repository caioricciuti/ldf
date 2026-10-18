#!/usr/bin/env bash
# The agreed evaluation runs, one stage per argument. Keys come from .env at
# the repo root (read by run_eval.py, never printed). Every stage resumes
# where it stopped and stops on its own at its --budget cap, so a stage can
# be re-run safely. Prices are USD per million tokens as of 2026-10-18.
#
#   eval/plan.sh opus-ldf      Opus 5.5, LDF only, 50-record bundles      (~$0.30)
#   eval/plan.sh haiku         Haiku 5.5, all formats, 200 and 500 records (~$1.50)
#   eval/plan.sh opus-200      Opus 5.5, all formats, flat_users_200       (cap $2.50)
#   eval/plan.sh deepseek      deepseek-chat, all formats, 200 and 500      (cap $1.20 each)
#   eval/plan.sh haiku-cap     Haiku 5.5, sparse_events_200, max_tokens 4096 (cap $0.50)
#   eval/plan.sh all           every stage above, in that order
#   eval/plan.sh score         regrade every responses file into results.<model>.md
set -euo pipefail
cd "$(dirname "$0")/.."

run() { python3 eval/run_eval.py --confirm-spend "$@"; }

OPUS=(--provider anthropic --model claude-opus-5-5 --effort low --price-input 4 --price-cached 0.20 --price-output 20)
HAIKU=(--provider anthropic --model claude-haiku-5-5 --effort low --price-input 0.10 --price-cached 0.01 --price-output 0.50)
DEEPSEEK=(--provider deepseek --model deepseek-chat --price-input 0.28 --price-cached 0.03 --price-output 0.42)

stage_opus_ldf() {
  for b in flat_users nested_orders sparse_events; do
    run --bundle "eval/out/$b" "${OPUS[@]}" --formats ldf --budget 0.50
  done
}
stage_haiku() {
  for b in flat_users_200 sparse_events_200 flat_users_500 sparse_events_500; do
    run --bundle "eval/out/$b" "${HAIKU[@]}" --budget 0.50
  done
}
stage_opus_200() {
  run --bundle eval/out/flat_users_200 "${OPUS[@]}" --budget 2.50
}
stage_haiku_cap() {
  # Same prompts as the haiku stage with four times the output cap, to tell
  # truncation from inability on the one cell where LDF lost (5 vs 2 T).
  run --bundle eval/out/sparse_events_200 "${HAIKU[@]}" --max-tokens 4096 --tag cap4096 --budget 0.50
}
stage_deepseek() {
  for b in flat_users_200 sparse_events_200 flat_users_500 sparse_events_500; do
    run --bundle "eval/out/$b" "${DEEPSEEK[@]}" --budget 1.20
  done
}
stage_score() {
  for r in eval/out/*/responses.*.jsonl; do
    bundle=$(dirname "$r")
    model=$(basename "$r" .jsonl | cut -d. -f3-)
    ./target/release/ldf-eval score --bundle "$bundle" --responses "$r" --out "$bundle/results.$model.md"
  done
}

[ $# -gt 0 ] || { sed -n '2,13p' "$0"; exit 1; }
for stage in "$@"; do
  case "$stage" in
    opus-ldf) stage_opus_ldf ;;
    haiku) stage_haiku ;;
    opus-200) stage_opus_200 ;;
    deepseek) stage_deepseek ;;
    haiku-cap) stage_haiku_cap ;;
    all) stage_opus_ldf; stage_haiku; stage_opus_200; stage_deepseek ;;
    score) stage_score ;;
    *) echo "unknown stage: $stage" >&2; exit 1 ;;
  esac
done
