#!/usr/bin/env python3
"""Send an ldf-eval bundle to an LLM provider and record responses.

Standard library only (urllib), so nothing to install. Keys come from the
environment: OPENAI_API_KEY or ANTHROPIC_API_KEY. Nothing is sent without
--confirm-spend; use --dry-run to see the exact request body first.

    python3 eval/run_eval.py --bundle eval/out/flat_users --provider anthropic \
        --model claude-opus-5-5 --dry-run
    python3 eval/run_eval.py --bundle eval/out/flat_users --provider openai \
        --model gpt-5 --max-prompts 20 --confirm-spend

Output: <bundle>/responses.<provider>.<model>.jsonl, one line per prompt, in
the schema `ldf-eval score` reads. Re-running resumes: prompts already in the
file are skipped.

Sampling settings are fixed per provider and recorded with every response:
OpenAI: temperature 0 unless --no-temperature (reasoning models reject it),
seed 0, max_completion_tokens. Anthropic: no temperature (current models
reject non-default sampling), max_tokens, optional output_config.effort.
"""

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request

OPENAI_URL = "https://api.openai.com/v1/chat/completions"
ANTHROPIC_URL = "https://api.anthropic.com/v1/messages"
ANTHROPIC_VERSION = "2023-06-01"


def read_jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def build_request(provider, model, prompt, max_tokens, args):
    if provider == "openai":
        body = {
            "model": model,
            "messages": [
                {"role": "system", "content": prompt["system"]},
                {"role": "user", "content": prompt["user"]},
            ],
            "max_completion_tokens": max_tokens,
            "seed": 0,
        }
        if args.reasoning_effort:
            body["reasoning_effort"] = args.reasoning_effort
        if not args.no_temperature:
            body["temperature"] = 0
        headers = {
            "Content-Type": "application/json",
            "Authorization": "Bearer " + os.environ.get("OPENAI_API_KEY", ""),
        }
        return OPENAI_URL, headers, body
    if provider == "anthropic":
        # The data block is identical for every task of one (dataset, format),
        # and prompts are ordered so those come consecutively. Marking it as a
        # cache breakpoint makes the repeated reads cost a fraction of a full
        # read. Caching does not change the model's answer; --no-cache turns
        # it off for an A/B check.
        content = prompt["user"]
        marker = "</data>\n\n"
        if not args.no_cache and marker in content:
            head, tail = content.split(marker, 1)
            content = [
                {"type": "text", "text": head + marker, "cache_control": {"type": "ephemeral"}},
                {"type": "text", "text": tail},
            ]
        body = {
            "model": model,
            "max_tokens": max_tokens,
            "system": prompt["system"],
            "messages": [{"role": "user", "content": content}],
        }
        if args.effort:
            body["output_config"] = {"effort": args.effort}
        headers = {
            "Content-Type": "application/json",
            "x-api-key": os.environ.get("ANTHROPIC_API_KEY", ""),
            "anthropic-version": ANTHROPIC_VERSION,
        }
        return ANTHROPIC_URL, headers, body
    raise SystemExit(f"unknown provider {provider}")


def parse_reply(provider, data):
    """Returns (text, finish_reason, input_tokens, output_tokens, cached_tokens, error).

    input_tokens is the full prompt size as the provider counts it (for
    Anthropic: uncached + cache writes + cache reads), so it is comparable
    across cached and uncached runs. cached_tokens says how much of it was
    served from cache.
    """
    if provider == "openai":
        choice = data["choices"][0]
        usage = data.get("usage", {})
        cached = (usage.get("prompt_tokens_details") or {}).get("cached_tokens")
        reasoning = (usage.get("completion_tokens_details") or {}).get("reasoning_tokens")
        content = choice["message"].get("content")
        if reasoning:
            # Keep the visible text; the reasoning count travels in the record.
            content = content if content is not None else ""
        return (
            content,
            choice.get("finish_reason"),
            usage.get("prompt_tokens"),
            usage.get("completion_tokens"),
            cached,
            None if content else "empty completion (reasoning_tokens=%s)" % reasoning,
        )
    stop = data.get("stop_reason")
    usage = data.get("usage", {})
    text = "".join(b.get("text", "") for b in data.get("content", []) if b.get("type") == "text")
    error = None
    if stop == "refusal":
        details = data.get("stop_details") or {}
        error = "refusal: " + str(details.get("category"))
    cache_read = usage.get("cache_read_input_tokens") or 0
    cache_write = usage.get("cache_creation_input_tokens") or 0
    uncached = usage.get("input_tokens")
    total_in = None if uncached is None else uncached + cache_read + cache_write
    return text, stop, total_in, usage.get("output_tokens"), cache_read or None, error


# Request fields a provider may reject for a given model (reasoning models
# refuse temperature, some refuse seed or reasoning_effort). On a 400 that
# names one of these, the field is dropped and the request retried once; the
# settings actually used are recorded with every response.
DROPPABLE = ("temperature", "seed", "reasoning_effort", "top_p")


def call(url, headers, body, timeout, retries):
    delay = 2.0
    dropped = []
    for attempt in range(retries + 1):
        payload = json.dumps(body).encode("utf-8")
        req = urllib.request.Request(url, data=payload, headers=headers, method="POST")
        started = time.monotonic()
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                raw = resp.read()
            latency = (time.monotonic() - started) * 1000.0
            return json.loads(raw), latency, None
        except urllib.error.HTTPError as e:
            text = e.read().decode("utf-8", "replace")
            if e.code == 400:
                culprit = next((f for f in DROPPABLE if f in body and f in text), None)
                if culprit and attempt < retries:
                    body.pop(culprit)
                    dropped.append(culprit)
                    print(f"  note: {culprit} rejected by the model, retrying without it", file=sys.stderr)
                    continue
            if e.code in (408, 409, 429) or e.code >= 500:
                if attempt < retries:
                    wait = e.headers.get("retry-after")
                    time.sleep(float(wait) if wait and wait.isdigit() else delay)
                    delay = min(delay * 2, 60)
                    continue
            return None, (time.monotonic() - started) * 1000.0, f"http {e.code}: {text[:500]}"
        except (urllib.error.URLError, TimeoutError, OSError) as e:
            if attempt < retries:
                time.sleep(delay)
                delay = min(delay * 2, 60)
                continue
            return None, (time.monotonic() - started) * 1000.0, f"transport: {e}"
    return None, 0.0, "exhausted retries"


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--bundle", required=True)
    p.add_argument("--provider", required=True, choices=["openai", "anthropic"])
    p.add_argument("--model", required=True, help="exact model id, recorded with every response")
    p.add_argument("--formats", help="comma separated subset of formats")
    p.add_argument("--max-prompts", type=int, help="stop after this many prompts (after resume filtering)")
    p.add_argument("--max-tokens", type=int, default=1024,
                   help="answers are short JSON values, but models show arithmetic for sums and means; "
                        "256 truncated every sum question in the first runs, 1024 leaves room")
    p.add_argument("--effort", choices=["low", "medium", "high", "xhigh", "max"], help="Anthropic output_config.effort")
    p.add_argument("--no-temperature", action="store_true", help="omit temperature (OpenAI reasoning models reject it; it is also dropped automatically on a 400)")
    p.add_argument("--reasoning-effort", choices=["none", "minimal", "low", "medium", "high", "xhigh", "max"],
                   help="OpenAI reasoning_effort for GPT-5+ models; reasoning tokens are billed as output")
    p.add_argument("--no-cache", action="store_true", help="Anthropic: do not mark the data block as a prompt-cache breakpoint")
    p.add_argument("--sleep", type=float, default=0.0, help="seconds between requests")
    p.add_argument("--timeout", type=float, default=300.0)
    p.add_argument("--retries", type=int, default=4)
    p.add_argument("--price-input", type=float, help="USD per 1M uncached input tokens, for the running cost line")
    p.add_argument("--price-cached", type=float, help="USD per 1M cached input tokens")
    p.add_argument("--price-output", type=float, help="USD per 1M output tokens (reasoning tokens included)")
    p.add_argument("--budget", type=float, help="stop sending once the running cost estimate reaches this many USD")
    p.add_argument("--dry-run", action="store_true", help="print the first request body and exit")
    p.add_argument("--confirm-spend", action="store_true", help="required to send anything")
    args = p.parse_args()

    prompts = read_jsonl(os.path.join(args.bundle, "prompts.jsonl"))
    manifest = json.load(open(os.path.join(args.bundle, "manifest.json"), encoding="utf-8"))
    if args.formats:
        keep = set(args.formats.split(","))
        prompts = [q for q in prompts if q["format"] in keep]
    out_path = os.path.join(args.bundle, f"responses.{args.provider}.{args.model}.jsonl")
    done = set()
    if os.path.exists(out_path):
        done = {r["prompt_id"] for r in read_jsonl(out_path)}
    todo = [q for q in prompts if q["prompt_id"] not in done]
    if args.max_prompts is not None:
        todo = todo[: args.max_prompts]
    total_tokens = sum(q.get("prompt_tokens_o200k") or 0 for q in todo)

    print(f"bundle: {manifest['dataset']} ({manifest['record_count']} records), formats: {', '.join(manifest['formats'])}")
    print(f"provider: {args.provider}, model: {args.model}, output: {out_path}")
    print(f"prompts to send: {len(todo)} ({len(done)} already done), "
          f"total prompt tokens (o200k_base, exact for GPT, indicative only for Claude): {total_tokens}")
    if not todo:
        return
    url, headers, body = build_request(args.provider, args.model, todo[0], args.max_tokens, args)
    if args.dry_run:
        shown = dict(body)
        print(json.dumps(shown, indent=2)[:4000])
        print("... (dry run, nothing sent)")
        return
    key_var = "OPENAI_API_KEY" if args.provider == "openai" else "ANTHROPIC_API_KEY"
    if not os.environ.get(key_var):
        raise SystemExit(f"{key_var} is not set")
    if not args.confirm_spend:
        raise SystemExit("refusing to send: pass --confirm-spend after reviewing the counts above")

    if args.budget is not None and None in (args.price_input, args.price_cached, args.price_output):
        raise SystemExit("--budget needs --price-input, --price-cached and --price-output")
    spent = 0.0

    def cost_of(rec):
        if None in (args.price_input, args.price_cached, args.price_output):
            return None
        cached = rec["cached_tokens"] or 0
        uncached = max((rec["input_tokens"] or 0) - cached, 0)
        return (uncached * args.price_input + cached * args.price_cached + (rec["output_tokens"] or 0) * args.price_output) / 1e6

    with open(out_path, "a", encoding="utf-8") as out:
        for i, q in enumerate(todo, 1):
            if args.budget is not None and spent >= args.budget:
                print(f"budget of ${args.budget:.2f} reached after {i - 1} prompts (${spent:.3f}); stopping. Re-run to resume.")
                break
            url, headers, body = build_request(args.provider, args.model, q, args.max_tokens, args)
            data, latency, err = call(url, headers, body, args.timeout, args.retries)
            rec = {
                "prompt_id": q["prompt_id"],
                "task_id": q["task_id"],
                "format": q["format"],
                "provider": args.provider,
                "model": args.model,
                "text": None,
                "finish_reason": None,
                "error": err,
                "input_tokens": None,
                "output_tokens": None,
                "cached_tokens": None,
                "latency_ms": latency,
                "settings": {k: v for k, v in body.items() if k not in ("messages", "system")},
            }
            if data is not None:
                text, finish, itok, otok, cached, perr = parse_reply(args.provider, data)
                rec.update(text=text, finish_reason=finish, input_tokens=itok, output_tokens=otok, cached_tokens=cached, error=perr)
            out.write(json.dumps(rec, ensure_ascii=False) + "\n")
            out.flush()
            status = rec["error"] or (rec["text"] or "")[:60].replace("\n", " ")
            c = cost_of(rec)
            cost_txt = ""
            if c is not None:
                spent += c
                cost_txt = f"  ${spent:.3f}"
            print(f"[{i}/{len(todo)}] {q['prompt_id']:<40} {latency:7.0f} ms  in={rec['input_tokens']} out={rec['output_tokens']}{cost_txt}  {status}")
            if args.sleep:
                time.sleep(args.sleep)
    if args.price_input is not None:
        print(f"estimated spend this invocation: ${spent:.3f} (uncached input at ${args.price_input}/M, cached at ${args.price_cached}/M, output at ${args.price_output}/M; Anthropic cache writes counted at the uncached rate)")


if __name__ == "__main__":
    sys.exit(main())
