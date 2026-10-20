# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | csv | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 7755 | 2962 ms |
| claude-opus-5-5 | json | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 13742 | 3468 ms |
| claude-opus-5-5 | json-pretty | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 18947 | 2716 ms |
| claude-opus-5-5 | jsonl | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 13942 | 2853 ms |
| claude-opus-5-5 | ldf | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 7778 | 2652 ms |
| claude-opus-5-5 | markdown | 15 | 93.3% | 14 | 0 | 0 | 0 | 1 | 0 | 0 | 9784 | 2557 ms |
| claude-opus-5-5 | toon | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 7961 | 2764 ms |
| claude-opus-5-5 | toon-tab | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 7965 | 3010 ms |
| claude-opus-5-5 | tsv | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 7756 | 2882 ms |
| claude-opus-5-5 | yaml | 15 | 86.7% | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 13957 | 2784 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | csv | lookup | 3 | 100.0% |
| claude-opus-5-5 | csv | filter | 3 | 100.0% |
| claude-opus-5-5 | csv | aggregate | 3 | 33.3% |
| claude-opus-5-5 | csv | compare | 3 | 100.0% |
| claude-opus-5-5 | csv | multi_record | 3 | 100.0% |
| claude-opus-5-5 | json | lookup | 3 | 100.0% |
| claude-opus-5-5 | json | filter | 3 | 100.0% |
| claude-opus-5-5 | json | aggregate | 3 | 33.3% |
| claude-opus-5-5 | json | compare | 3 | 100.0% |
| claude-opus-5-5 | json | multi_record | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | lookup | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | filter | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | aggregate | 3 | 33.3% |
| claude-opus-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | multi_record | 3 | 100.0% |
| claude-opus-5-5 | jsonl | lookup | 3 | 100.0% |
| claude-opus-5-5 | jsonl | filter | 3 | 100.0% |
| claude-opus-5-5 | jsonl | aggregate | 3 | 33.3% |
| claude-opus-5-5 | jsonl | compare | 3 | 100.0% |
| claude-opus-5-5 | jsonl | multi_record | 3 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 3 | 100.0% |
| claude-opus-5-5 | ldf | filter | 3 | 100.0% |
| claude-opus-5-5 | ldf | aggregate | 3 | 33.3% |
| claude-opus-5-5 | ldf | compare | 3 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 3 | 100.0% |
| claude-opus-5-5 | markdown | lookup | 3 | 100.0% |
| claude-opus-5-5 | markdown | filter | 3 | 100.0% |
| claude-opus-5-5 | markdown | aggregate | 3 | 66.7% |
| claude-opus-5-5 | markdown | compare | 3 | 100.0% |
| claude-opus-5-5 | markdown | multi_record | 3 | 100.0% |
| claude-opus-5-5 | toon | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon | filter | 3 | 100.0% |
| claude-opus-5-5 | toon | aggregate | 3 | 33.3% |
| claude-opus-5-5 | toon | compare | 3 | 100.0% |
| claude-opus-5-5 | toon | multi_record | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | filter | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | aggregate | 3 | 33.3% |
| claude-opus-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | multi_record | 3 | 100.0% |
| claude-opus-5-5 | tsv | lookup | 3 | 100.0% |
| claude-opus-5-5 | tsv | filter | 3 | 100.0% |
| claude-opus-5-5 | tsv | aggregate | 3 | 33.3% |
| claude-opus-5-5 | tsv | compare | 3 | 100.0% |
| claude-opus-5-5 | tsv | multi_record | 3 | 100.0% |
| claude-opus-5-5 | yaml | lookup | 3 | 100.0% |
| claude-opus-5-5 | yaml | filter | 3 | 100.0% |
| claude-opus-5-5 | yaml | aggregate | 3 | 33.3% |
| claude-opus-5-5 | yaml | compare | 3 | 100.0% |
| claude-opus-5-5 | yaml | multi_record | 3 | 100.0% |
