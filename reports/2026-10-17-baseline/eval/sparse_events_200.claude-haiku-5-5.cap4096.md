# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | csv | 18 | 88.9% | 16 | 1 | 0 | 0 | 1 | 0 | 0 | 9383 | 2736 ms |
| claude-haiku-5-5 | json | 18 | 94.4% | 17 | 1 | 0 | 0 | 0 | 0 | 0 | 15317 | 1956 ms |
| claude-haiku-5-5 | json-pretty | 18 | 94.4% | 17 | 1 | 0 | 0 | 0 | 0 | 0 | 20668 | 2055 ms |
| claude-haiku-5-5 | jsonl | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 15517 | 2464 ms |
| claude-haiku-5-5 | ldf | 18 | 72.2% | 13 | 1 | 2 | 0 | 2 | 0 | 0 | 9661 | 3836 ms |
| claude-haiku-5-5 | markdown | 18 | 94.4% | 17 | 0 | 1 | 0 | 0 | 0 | 0 | 12683 | 2310 ms |
| claude-haiku-5-5 | toon | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 16245 | 2241 ms |
| claude-haiku-5-5 | toon-tab | 18 | 94.4% | 17 | 1 | 0 | 0 | 0 | 0 | 0 | 16249 | 2174 ms |
| claude-haiku-5-5 | tsv | 18 | 83.3% | 15 | 2 | 1 | 0 | 0 | 0 | 0 | 7971 | 2807 ms |
| claude-haiku-5-5 | yaml | 18 | 88.9% | 16 | 2 | 0 | 0 | 0 | 0 | 0 | 15717 | 2182 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | csv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | csv | filter | 3 | 66.7% |
| claude-haiku-5-5 | csv | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | csv | compare | 3 | 100.0% |
| claude-haiku-5-5 | csv | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | csv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json | lookup | 3 | 100.0% |
| claude-haiku-5-5 | json | filter | 3 | 100.0% |
| claude-haiku-5-5 | json | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | json | compare | 3 | 100.0% |
| claude-haiku-5-5 | json | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | lookup | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | filter | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | lookup | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | filter | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | aggregate | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | compare | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | ldf | lookup | 3 | 100.0% |
| claude-haiku-5-5 | ldf | filter | 3 | 66.7% |
| claude-haiku-5-5 | ldf | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | ldf | compare | 3 | 33.3% |
| claude-haiku-5-5 | ldf | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | ldf | multi_record | 3 | 66.7% |
| claude-haiku-5-5 | markdown | lookup | 3 | 100.0% |
| claude-haiku-5-5 | markdown | filter | 3 | 100.0% |
| claude-haiku-5-5 | markdown | aggregate | 3 | 100.0% |
| claude-haiku-5-5 | markdown | compare | 3 | 66.7% |
| claude-haiku-5-5 | markdown | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | markdown | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon | lookup | 3 | 100.0% |
| claude-haiku-5-5 | toon | filter | 3 | 100.0% |
| claude-haiku-5-5 | toon | aggregate | 3 | 100.0% |
| claude-haiku-5-5 | toon | compare | 3 | 100.0% |
| claude-haiku-5-5 | toon | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | toon | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | lookup | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | filter | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | tsv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | tsv | filter | 3 | 66.7% |
| claude-haiku-5-5 | tsv | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | tsv | compare | 3 | 66.7% |
| claude-haiku-5-5 | tsv | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | tsv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | yaml | lookup | 3 | 100.0% |
| claude-haiku-5-5 | yaml | filter | 3 | 100.0% |
| claude-haiku-5-5 | yaml | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | yaml | compare | 3 | 100.0% |
| claude-haiku-5-5 | yaml | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | yaml | multi_record | 3 | 100.0% |
