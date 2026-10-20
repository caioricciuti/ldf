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
#   eval/plan.sh sparse-sweep-build     sparse_events at threshold 0.75 (free)
#   eval/plan.sh sparse-sweep-deepseek  deepseek-chat on it (~$0.04, cap $0.10)
#   eval/plan.sh stage-c-build          Stage C bundles on held-out data (free)
#   eval/plan.sh stage-c-opus           Opus 5.5 on them, distinct renderings only (~$9-17, cap $17)
#   eval/plan.sh stage-c10-build        Stage C at 10 questions per kind (free)
#   eval/plan.sh stage-c10-opus         Opus 5.5 on it, JSON and LDF variants only (~$5.00-6.60, cap $7)
#   eval/plan.sh stage-c10-sonnet       Sonnet 5.5 on it, same prompts (~$4.30-6.50, cap $7)
#   eval/plan.sh large-build            1 000 held-out records, no arithmetic kinds (free)
#   eval/plan.sh large-opus             Opus 5.5 on it (cap $16)
#   eval/plan.sh large-sonnet           Sonnet 5.5 on it (cap $13)
#   eval/plan.sh stage-c-sonnet         Sonnet 5.5 on them (~$7.50-11.50, cap $12)
#   eval/plan.sh stage-c-gpt            GPT on them; needs GPT_MODEL and GPT_PRICE_IN/CACHED/OUT set (cap $GPT_BUDGET)
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
# Sparse threshold sweep for DeepSeek (Stage B finding 3): the Stage B
# sparse_events records and questions with v0.1 at threshold 0.75 (1.0
# gives the same document here: no key between 60 and 100 percent fill).
# Compared against the Stage B answers for json, toon, ldf (0.5), ldf-v0 (0).
sparse_sweep_build() {
  local s75='d=comma h=typed q?=1 o=dotted a=child-key sp=75 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  ./target/release/ldf-eval build --input fixtures/medium/sparse_events.json --records 200 --per-kind 15 --seed 1 \
    --formats json,ldf --candidate "ldf-s75=$s75" --out eval/out/sparse-sweep/sparse_events
}
stage_sparse_sweep_deepseek() {
  run --bundle eval/out/sparse-sweep/sparse_events "${DEEPSEEK[@]}" --formats ldf-s75 --budget 0.10
}
# Stage C (discovery step 4): finalists on held-out data. fixtures/holdout
# (ldf-gen --seed 20261118) with question seed 2, so neither the records
# nor the questions were seen when the finalists were picked.
STAGE_C_DATASETS=(flat_users nested_orders sparse_events mixed_types)
stage_c_build() {
  local per_kind=${1:-15} dir=${2:-eval/out/stage-c}
  local s75='d=comma h=typed q?=1 o=dotted a=child-key sp=75 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  local h='d=comma h=typed q?=1 o=dotted a=hoisted sp=50 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  local hs75='d=comma h=typed q?=1 o=dotted a=hoisted sp=75 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  for b in "${STAGE_C_DATASETS[@]}"; do
    ./target/release/ldf-eval build --input "fixtures/holdout/medium/$b.json" --records 200 --per-kind "$per_kind" --seed 2 \
      --formats json,toon,ldf --candidate "ldf-s75=$s75" --candidate "ldf-hoisted=$h" --candidate "ldf-hs75=$hs75" \
      --out "$dir/$b"
  done
}
# Stage C runs: only renderings that differ are sent (ldf-s75 equals ldf on
# flat_users and mixed_types; ldf-hoisted equals ldf and ldf-hs75 equals
# ldf-s75 on sparse_events). 1 789 prompts, 20.6M o200k input tokens, about
# 27.6M Claude tokens (1.34x o200k, measured on the Opus responses), almost
# all prompt-cache reads. The cap is per bundle; the four caps sum to the
# stage total in the header.
STAGE_C=(
  "flat_users json,toon,ldf"
  "nested_orders json,toon,ldf,ldf-s75,ldf-hoisted,ldf-hs75"
  "sparse_events json,toon,ldf,ldf-s75"
  "mixed_types json,toon,ldf"
)
SONNET=(--provider anthropic --model claude-sonnet-5-5 --effort low --price-input 2 --price-cached 0.20 --price-output 10)
stage_c() {
  local dir=$1 total=$2; shift 2
  for entry in "${STAGE_C[@]}"; do
    read -r b formats <<< "$entry"
    # Budget split in proportion to the bundle's share of input tokens.
    local share
    case "$b" in
      flat_users) share=0.10 ;; nested_orders) share=0.60 ;; sparse_events) share=0.24 ;; mixed_types) share=0.06 ;;
    esac
    run --bundle "$dir/$b" "$@" --formats "$formats" --max-tokens 4096 \
      --budget "$(python3 -c "print(round($total * $share, 2))")"
  done
}
stage_stage_c_opus() { stage_c eval/out/stage-c 17 "${OPUS[@]}"; }
# Stage C as run (agreed 2026-10-19): Opus on eval/out/stage-c10, 10
# questions per kind (8 left mixed_types at 47, under the selector minimum of
# 50 paired questions). Only what the selector and the v0.2 question need:
# no TOON (ldf-mcp never sends it), hoisted at 0.5 on nested_orders only.
# 864 prompts, about $5.00-6.60 at 80-170 output tokens per answer; cap $7,
# split by each bundle's share of the estimate.
STAGE_C10=(
  "flat_users json,ldf 0.13"
  "nested_orders json,ldf,ldf-s75,ldf-hoisted 0.50"
  "sparse_events json,ldf,ldf-s75 0.27"
  "mixed_types json,ldf 0.10"
)
stage_c10() {
  local total=$1; shift
  for entry in "${STAGE_C10[@]}"; do
    read -r b formats share <<< "$entry"
    run --bundle "eval/out/stage-c10/$b" "$@" --formats "$formats" --max-tokens 4096 \
      --budget "$(python3 -c "print(round($total * $share, 2))")"
  done
}
stage_stage_c10_opus() { stage_c10 7 "${OPUS[@]}"; }
# Sonnet 5.5 on the same 864 prompts (2026-10-20). The Opus run's own usage
# repriced at Sonnet rates is $4.26; cache reads cost the same on both, so
# the spread is output length and tokenizer: about $4.30-6.50, cap $7.
stage_stage_c10_sonnet() { stage_c10 7 "${SONNET[@]}"; }
# Large (2026-10-20): 1 000 held-out records, so the selector has evidence
# above 200 rows. Only kinds a model can answer at that size without long
# arithmetic: no aggregate, aggregate_where or child_aggregate (sums and
# means failed in every format at 500 records). flat_users runs out of
# distinct questions, so it asks 20 per kind (64 questions); the others 10 (52 to 68 per
# dataset). Each model gets only the layouts its selector could pick:
# hoisted nested for Opus, threshold 0.75 nested for Sonnet, no 0.75 on
# sparse (both models rejected it at 200).
LARGE_KINDS=lookup,filter,compare,multi_record,rank,missing_value,nested_extract,child_count
large_build() {
  local s75='d=comma h=typed q?=1 o=dotted a=child-key sp=75 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  local h='d=comma h=typed q?=1 o=dotted a=hoisted sp=50 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0'
  for b in "${STAGE_C_DATASETS[@]}"; do
    local pk=10
    [ "$b" = flat_users ] && pk=20
    ./target/release/ldf-eval build --input "fixtures/holdout/medium/$b.json" --records 1000 --per-kind "$pk" --seed 2 \
      --kinds "$LARGE_KINDS" --formats json,ldf --candidate "ldf-s75=$s75" --candidate "ldf-hoisted=$h" \
      --out "eval/out/large/$b"
  done
}
large_run() {
  local total=$1 nested=$2; shift 2
  local entry b formats share
  for entry in "flat_users json,ldf 0.14" "nested_orders $nested 0.57" "sparse_events json,ldf 0.22" "mixed_types json,ldf 0.07"; do
    read -r b formats share <<< "$entry"
    run --bundle "eval/out/large/$b" "$@" --formats "$formats" --max-tokens 4096 \
      --budget "$(python3 -c "print(round($total * $share, 2))")"
  done
}
stage_large_opus() { large_run 16 json,ldf,ldf-hoisted "${OPUS[@]}"; }
stage_large_sonnet() { large_run 13 json,ldf,ldf-s75 "${SONNET[@]}"; }
stage_stage_c_sonnet() { stage_c eval/out/stage-c 12 "${SONNET[@]}"; }
stage_stage_c_gpt() {
  : "${GPT_MODEL:?set GPT_MODEL}" "${GPT_PRICE_IN:?set GPT_PRICE_IN (USD per M input)}" \
    "${GPT_PRICE_CACHED:?set GPT_PRICE_CACHED}" "${GPT_PRICE_OUT:?set GPT_PRICE_OUT}" "${GPT_BUDGET:?set GPT_BUDGET (USD, whole stage)}"
  stage_c eval/out/stage-c "$GPT_BUDGET" --provider openai --model "$GPT_MODEL" --reasoning-effort low \
    --price-input "$GPT_PRICE_IN" --price-cached "$GPT_PRICE_CACHED" --price-output "$GPT_PRICE_OUT"
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
  for r in eval/out/*/responses.*.jsonl eval/out/stage-b/*/responses.*.jsonl eval/out/sparse-sweep/*/responses.*.jsonl eval/out/stage-c/*/responses.*.jsonl eval/out/stage-c10/*/responses.*.jsonl eval/out/large/*/responses.*.jsonl; do
    [ -e "$r" ] || continue
    bundle=$(dirname "$r")
    model=$(basename "$r" .jsonl | cut -d. -f3-)
    ./target/release/ldf-eval score --bundle "$bundle" --responses "$r" --out "$bundle/results.$model.md"
  done
}

[ $# -gt 0 ] || { sed -n '2,25p' "$0"; exit 1; }
for stage in "$@"; do
  case "$stage" in
    opus-ldf) stage_opus_ldf ;;
    haiku) stage_haiku ;;
    opus-200) stage_opus_200 ;;
    deepseek) stage_deepseek ;;
    haiku-cap) stage_haiku_cap ;;
    haiku-sparse) stage_haiku_sparse ;;
    stage-b-build) stage_b_build ;;
    sparse-sweep-build) sparse_sweep_build ;;
    stage-c-build) stage_c_build ;;
    stage-c-opus) stage_stage_c_opus ;;
    stage-c10-build) stage_c_build 10 eval/out/stage-c10 ;;
    stage-c10-opus) stage_stage_c10_opus ;;
    stage-c10-sonnet) stage_stage_c10_sonnet ;;
    large-build) large_build ;;
    large-opus) stage_large_opus ;;
    large-sonnet) stage_large_sonnet ;;
    stage-c-sonnet) stage_stage_c_sonnet ;;
    stage-c-gpt) stage_stage_c_gpt ;;
    sparse-sweep-deepseek) stage_sparse_sweep_deepseek ;;
    stage-b-haiku) stage_stage_b_haiku ;;
    stage-b-deepseek) stage_stage_b_deepseek ;;
    all) stage_opus_ldf; stage_haiku; stage_opus_200; stage_deepseek ;;
    score) stage_score ;;
    *) echo "unknown stage: $stage" >&2; exit 1 ;;
  esac
done
