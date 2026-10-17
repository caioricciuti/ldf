#!/usr/bin/env python3
"""Exact Claude token counts for the encoded documents of an ldf-bench run.

Anthropic does not publish its tokenizer, so counts come from the
count_tokens endpoint, which is free of charge but needs ANTHROPIC_API_KEY.
Counts are model specific; pass the model you will evaluate with.

    python3 eval/count_tokens_claude.py --results results/medium --model claude-opus-5-5

Reads <results>/encoded/<dataset>.<format>.<ext> (written by `ldf-bench
tokens`), writes <results>/tokens-claude.<model>.json and prints a table that
merges the Claude counts with the o200k_base counts from tokens.json.
"""

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request

URL = "https://api.anthropic.com/v1/messages/count_tokens"


def count(model, text, key, timeout=120):
    body = json.dumps({"model": model, "messages": [{"role": "user", "content": text}]}).encode("utf-8")
    req = urllib.request.Request(
        URL,
        data=body,
        method="POST",
        headers={"Content-Type": "application/json", "x-api-key": key, "anthropic-version": "2023-06-01"},
    )
    for attempt in range(5):
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                return json.loads(resp.read())["input_tokens"], None
        except urllib.error.HTTPError as e:
            msg = e.read().decode("utf-8", "replace")
            if e.code == 429 or e.code >= 500:
                wait = e.headers.get("retry-after")
                time.sleep(float(wait) if wait and wait.isdigit() else 2.0 * (attempt + 1))
                continue
            return None, f"http {e.code}: {msg[:300]}"
        except (urllib.error.URLError, OSError) as e:
            time.sleep(2.0 * (attempt + 1))
            last = str(e)
    return None, f"failed after retries: {last}"


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--results", required=True, help="an ldf-bench tokens results directory")
    p.add_argument("--model", required=True)
    p.add_argument("--dry-run", action="store_true", help="list files and sizes, send nothing")
    args = p.parse_args()

    enc_dir = os.path.join(args.results, "encoded")
    files = sorted(f for f in os.listdir(enc_dir) if not f.startswith("."))
    print(f"{len(files)} encoded documents in {enc_dir}, total {sum(os.path.getsize(os.path.join(enc_dir, f)) for f in files)} bytes")
    if args.dry_run:
        for f in files:
            print(f"  {f}  {os.path.getsize(os.path.join(enc_dir, f))} bytes")
        return
    key = os.environ.get("ANTHROPIC_API_KEY")
    if not key:
        raise SystemExit("ANTHROPIC_API_KEY is not set")

    results = {"model": args.model, "endpoint": URL, "counts": {}}
    for f in files:
        with open(os.path.join(enc_dir, f), encoding="utf-8") as fh:
            text = fh.read()
        tokens, err = count(args.model, text, key)
        results["counts"][f] = {"bytes": len(text.encode("utf-8")), "input_tokens": tokens, "error": err}
        print(f"  {f:<50} {tokens if tokens is not None else err}")
    out = os.path.join(args.results, f"tokens-claude.{args.model}.json")
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(results, fh, indent=2)
    print(f"wrote {out}")

    tokens_json = os.path.join(args.results, "tokens.json")
    if os.path.exists(tokens_json):
        run = json.load(open(tokens_json, encoding="utf-8"))
        print("\n| dataset | format | bytes | o200k_base | claude | claude bytes/token |")
        print("|---|---|---:|---:|---:|---:|")
        for ds in run["datasets"]:
            for ft in ds["tokens"]:
                if ft.get("unsupported"):
                    continue
                match = [k for k in results["counts"] if k.startswith(f"{ds['name']}.{ft['format']}.")]
                if not match:
                    continue
                c = results["counts"][match[0]]
                o200k = next((x["tokens"] for x in ft["counts"] if x["tokenizer"] == "o200k_base"), None)
                t = c["input_tokens"]
                bpt = f"{c['bytes'] / t:.2f}" if t else "-"
                print(f"| {ds['name']} | {ft['format']} | {ft['bytes']} | {o200k} | {t} | {bpt} |")


if __name__ == "__main__":
    sys.exit(main())
