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
#   eval/plan.sh haiku-sparse  Haiku 5.5, sparse_events_200_sparse (ldf-s25/s50/s75), max_tokens 4096 (cap $0.50)
#   eval/plan.sh stage-b-build     build the Stage B bundles (free, deterministic)
#   eval/plan.sh stage-b-haiku     Haiku 5.5, Stage B bundles, max_tokens 4096 (~$1.10, cap $1.60)
#   eval/plan.sh stage-b-deepseek  deepseek-chat, Stage B bundles             (~$1.25, cap $1.80)
#   eval/plan.sh all           the first four stages above, in that order
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
stage_haiku_sparse() {
  # The sparse-key rule at three thresholds, same tasks and cap as haiku-cap.
  run --bundle eval/out/sparse_events_200_sparse "${HAIKU[@]}" --max-tokens 4096 --budget 0.50
}
# Stage B (discovery step 3): eval/out/stage-b, 200 records of the seed-1
# medium fixtures, 15 questions per kind, LDF v0.1 and v0 plus three
# ldf-discover candidates (ldf-hoisted, ldf-tab, ldf-global; ids in the
# manifests). Only renderings that differ are sent: a candidate identical
# to ldf on a dataset is left out.
STAGE_B=(
  "flat_users json,csv,toon,ldf,ldf-tab,ldf-global"
  "nested_orders json,toon,ldf,ldf-hoisted,ldf-tab,ldf-global"
  "sparse_events json,csv,toon,ldf,ldf-v0,ldf-tab,ldf-global"
  "mixed_types json,toon,ldf,ldf-v0,ldf-tab,ldf-global"
)
stage_b_build() {
  local h='d=comma h=typed q?=1 o=dotted a=hoisted sp=50 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  local t='d=tab h=typed q?=1 o=dotted a=child-key sp=50 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  local g='d=comma h=untyped q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=once fl=keep b=words tr=1'
  for entry in "${STAGE_B[@]}"; do
    read -r b _ <<< "$entry"
    ./target/release/ldf-eval build --input "fixtures/medium/$b.json" --records 200 --per-kind 15 --seed 1 \
      --formats json,csv,toon,ldf,ldf-v0 --candidate "ldf-hoisted=$h" --candidate "ldf-tab=$t" \
      --candidate "ldf-global=$g" --out "eval/out/stage-b/$b"
  done
}
stage_b() {
  local cap=$1; shift
  for entry in "${STAGE_B[@]}"; do
    read -r b formats <<< "$entry"
    run --bundle "eval/out/stage-b/$b" "$@" --formats "$formats" --budget "$cap"
  done
}
stage_stage_b_haiku() { stage_b 0.40 "${HAIKU[@]}" --max-tokens 4096; }
stage_stage_b_deepseek() { stage_b 0.45 "${DEEPSEEK[@]}"; }
stage_deepseek() {
  for b in flat_users_200 sparse_events_200 flat_users_500 sparse_events_500; do
    run --bundle "eval/out/$b" "${DEEPSEEK[@]}" --budget 1.20
  done
}
stage_score() {
  for r in eval/out/*/responses.*.jsonl eval/out/stage-b/*/responses.*.jsonl; do
    [ -e "$r" ] || continue
    bundle=$(dirname "$r")
    model=$(basename "$r" .jsonl | cut -d. -f3-)
    ./target/release/ldf-eval score --bundle "$bundle" --responses "$r" --out "$bundle/results.$model.md"
  done
}

[ $# -gt 0 ] || { sed -n '2,17p' "$0"; exit 1; }
for stage in "$@"; do
  case "$stage" in
    opus-ldf) stage_opus_ldf ;;
    haiku) stage_haiku ;;
    opus-200) stage_opus_200 ;;
    deepseek) stage_deepseek ;;
    haiku-cap) stage_haiku_cap ;;
    haiku-sparse) stage_haiku_sparse ;;
    stage-b-build) stage_b_build ;;
    stage-b-haiku) stage_stage_b_haiku ;;
    stage-b-deepseek) stage_stage_b_deepseek ;;
    all) stage_opus_ldf; stage_haiku; stage_opus_200; stage_deepseek ;;
    score) stage_score ;;
    *) echo "unknown stage: $stage" >&2; exit 1 ;;
  esac
done
