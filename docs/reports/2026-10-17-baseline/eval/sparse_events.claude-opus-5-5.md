# LLM comprehension results

| model | format | n | accuracy | correct | wrong | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | csv | 18 | 94.4% | 17 | 0 | 0 | 1 | 0 | 0 | 2458 | 1890 ms |
| claude-opus-5-5 | json | 18 | 94.4% | 17 | 0 | 0 | 1 | 0 | 0 | 3878 | 1909 ms |
| claude-opus-5-5 | json-pretty | 18 | 94.4% | 17 | 0 | 0 | 1 | 0 | 0 | 5201 | 1917 ms |
| claude-opus-5-5 | jsonl | 18 | 94.4% | 17 | 0 | 0 | 1 | 0 | 0 | 3928 | 1939 ms |
| claude-opus-5-5 | markdown | 18 | 94.4% | 17 | 0 | 0 | 1 | 0 | 0 | 3331 | 1891 ms |
| claude-opus-5-5 | toon | 18 | 94.4% | 17 | 0 | 0 | 1 | 0 | 0 | 4104 | 1890 ms |
| claude-opus-5-5 | toon-tab | 18 | 94.4% | 17 | 0 | 0 | 1 | 0 | 0 | 4108 | 1954 ms |
| claude-opus-5-5 | tsv | 18 | 88.9% | 16 | 1 | 0 | 1 | 0 | 0 | 2097 | 2254 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | csv | lookup | 3 | 100.0% |
| claude-opus-5-5 | csv | filter | 3 | 100.0% |
| claude-opus-5-5 | csv | aggregate | 3 | 66.7% |
| claude-opus-5-5 | csv | compare | 3 | 100.0% |
| claude-opus-5-5 | csv | missing_value | 3 | 100.0% |
| claude-opus-5-5 | csv | multi_record | 3 | 100.0% |
| claude-opus-5-5 | json | lookup | 3 | 100.0% |
| claude-opus-5-5 | json | filter | 3 | 100.0% |
| claude-opus-5-5 | json | aggregate | 3 | 66.7% |
| claude-opus-5-5 | json | compare | 3 | 100.0% |
| claude-opus-5-5 | json | missing_value | 3 | 100.0% |
| claude-opus-5-5 | json | multi_record | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | lookup | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | filter | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | aggregate | 3 | 66.7% |
| claude-opus-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | missing_value | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | multi_record | 3 | 100.0% |
| claude-opus-5-5 | jsonl | lookup | 3 | 100.0% |
| claude-opus-5-5 | jsonl | filter | 3 | 100.0% |
| claude-opus-5-5 | jsonl | aggregate | 3 | 66.7% |
| claude-opus-5-5 | jsonl | compare | 3 | 100.0% |
| claude-opus-5-5 | jsonl | missing_value | 3 | 100.0% |
| claude-opus-5-5 | jsonl | multi_record | 3 | 100.0% |
| claude-opus-5-5 | markdown | lookup | 3 | 100.0% |
| claude-opus-5-5 | markdown | filter | 3 | 100.0% |
| claude-opus-5-5 | markdown | aggregate | 3 | 66.7% |
| claude-opus-5-5 | markdown | compare | 3 | 100.0% |
| claude-opus-5-5 | markdown | missing_value | 3 | 100.0% |
| claude-opus-5-5 | markdown | multi_record | 3 | 100.0% |
| claude-opus-5-5 | toon | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon | filter | 3 | 100.0% |
| claude-opus-5-5 | toon | aggregate | 3 | 66.7% |
| claude-opus-5-5 | toon | compare | 3 | 100.0% |
| claude-opus-5-5 | toon | missing_value | 3 | 100.0% |
| claude-opus-5-5 | toon | multi_record | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | filter | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | aggregate | 3 | 66.7% |
| claude-opus-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | missing_value | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | multi_record | 3 | 100.0% |
| claude-opus-5-5 | tsv | lookup | 3 | 100.0% |
| claude-opus-5-5 | tsv | filter | 3 | 100.0% |
| claude-opus-5-5 | tsv | aggregate | 3 | 66.7% |
| claude-opus-5-5 | tsv | compare | 3 | 66.7% |
| claude-opus-5-5 | tsv | missing_value | 3 | 100.0% |
| claude-opus-5-5 | tsv | multi_record | 3 | 100.0% |
