"""The `minified` arm of eval/db_eval.py: a stdio MCP proxy that forwards
every message unchanged except the text blocks of tool results that parse as
JSON, which it re-serializes without whitespace. As `ldf-mcp proxy` does,
a `structuredContent` that is exactly `{"result": S}` with S the original
text (FastMCP's wrapping, which Claude Code shows the model) gets the same
minified text. Nothing else: no primer, no LDF, `isError` results
untouched. Standard library only.

    python3 -I eval/minify_proxy.py -- COMMAND [ARGS...]
"""
import json, subprocess, sys, threading


def minify(msg):
    res = msg.get('result')
    if not isinstance(res, dict) or res.get('isError') or not isinstance(res.get('content'), list):
        return None
    changed = False
    sc = res.get('structuredContent')
    wrapped = sc['result'] if isinstance(sc, dict) and len(sc) == 1 and isinstance(sc.get('result'), str) else None
    for block in res['content']:
        text = block.get('text') if isinstance(block, dict) and block.get('type') == 'text' else None
        if not isinstance(text, str) or text.lstrip()[:1] not in ('{', '['):
            continue
        try:
            mini = json.dumps(json.loads(text), separators=(',', ':'), ensure_ascii=False)
        except ValueError:
            continue
        if len(mini) < len(text):
            if wrapped is not None and text == wrapped:
                sc['result'] = mini
                wrapped = None
            block['text'] = mini
            changed = True
    return msg if changed else None


def main():
    argv = sys.argv[sys.argv.index('--') + 1:]
    child = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def upstream():
        for line in sys.stdin:
            child.stdin.write(line)
            child.stdin.flush()
        child.stdin.close()

    threading.Thread(target=upstream, daemon=True).start()
    for line in child.stdout:
        try:
            out = minify(json.loads(line))
        except ValueError:
            out = None
        sys.stdout.write(line if out is None else json.dumps(out, ensure_ascii=False) + '\n')
        sys.stdout.flush()
    sys.exit(child.wait())


if __name__ == '__main__':
    main()
