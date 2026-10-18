# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | csv | 15 | 93.3% | 14 | 1 | 0 | 0 | 0 | 0 | 0 | 2065 | 1540 ms |
| claude-opus-5-5 | json | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 3548 | 1607 ms |
| claude-opus-5-5 | json-pretty | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 4853 | 1544 ms |
| claude-opus-5-5 | jsonl | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 3598 | 1621 ms |
| claude-opus-5-5 | ldf | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 2084 | 2019 ms |
| claude-opus-5-5 | markdown | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 2594 | 1677 ms |
| claude-opus-5-5 | toon | 15 | 93.3% | 14 | 1 | 0 | 0 | 0 | 0 | 0 | 2121 | 1667 ms |
| claude-opus-5-5 | toon-tab | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 2125 | 1764 ms |
| claude-opus-5-5 | tsv | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 2066 | 1715 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | csv | lookup | 3 | 100.0% |
| claude-opus-5-5 | csv | filter | 3 | 100.0% |
| claude-opus-5-5 | csv | aggregate | 3 | 100.0% |
| claude-opus-5-5 | csv | compare | 3 | 66.7% |
| claude-opus-5-5 | csv | multi_record | 3 | 100.0% |
| claude-opus-5-5 | json | lookup | 3 | 100.0% |
| claude-opus-5-5 | json | filter | 3 | 100.0% |
| claude-opus-5-5 | json | aggregate | 3 | 100.0% |
| claude-opus-5-5 | json | compare | 3 | 100.0% |
| claude-opus-5-5 | json | multi_record | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | lookup | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | filter | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | aggregate | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | multi_record | 3 | 100.0% |
| claude-opus-5-5 | jsonl | lookup | 3 | 100.0% |
| claude-opus-5-5 | jsonl | filter | 3 | 100.0% |
| claude-opus-5-5 | jsonl | aggregate | 3 | 100.0% |
| claude-opus-5-5 | jsonl | compare | 3 | 100.0% |
| claude-opus-5-5 | jsonl | multi_record | 3 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 3 | 100.0% |
| claude-opus-5-5 | ldf | filter | 3 | 100.0% |
| claude-opus-5-5 | ldf | aggregate | 3 | 100.0% |
| claude-opus-5-5 | ldf | compare | 3 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 3 | 100.0% |
| claude-opus-5-5 | markdown | lookup | 3 | 100.0% |
| claude-opus-5-5 | markdown | filter | 3 | 100.0% |
| claude-opus-5-5 | markdown | aggregate | 3 | 100.0% |
| claude-opus-5-5 | markdown | compare | 3 | 100.0% |
| claude-opus-5-5 | markdown | multi_record | 3 | 100.0% |
| claude-opus-5-5 | toon | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon | filter | 3 | 100.0% |
| claude-opus-5-5 | toon | aggregate | 3 | 100.0% |
| claude-opus-5-5 | toon | compare | 3 | 66.7% |
| claude-opus-5-5 | toon | multi_record | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | filter | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | aggregate | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | multi_record | 3 | 100.0% |
| claude-opus-5-5 | tsv | lookup | 3 | 100.0% |
| claude-opus-5-5 | tsv | filter | 3 | 100.0% |
| claude-opus-5-5 | tsv | aggregate | 3 | 100.0% |
| claude-opus-5-5 | tsv | compare | 3 | 100.0% |
| claude-opus-5-5 | tsv | multi_record | 3 | 100.0% |
