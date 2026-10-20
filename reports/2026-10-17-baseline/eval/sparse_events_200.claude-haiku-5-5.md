# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | csv | 18 | 83.3% | 15 | 0 | 0 | 0 | 3 | 0 | 0 | 9383 | 2286 ms |
| claude-haiku-5-5 | json | 18 | 88.9% | 16 | 0 | 0 | 0 | 2 | 0 | 0 | 15317 | 1676 ms |
| claude-haiku-5-5 | json-pretty | 18 | 88.9% | 16 | 0 | 0 | 0 | 2 | 0 | 0 | 20668 | 1761 ms |
| claude-haiku-5-5 | jsonl | 18 | 88.9% | 16 | 0 | 0 | 0 | 2 | 0 | 0 | 15517 | 1699 ms |
| claude-haiku-5-5 | ldf | 18 | 66.7% | 12 | 0 | 1 | 0 | 5 | 0 | 0 | 9661 | 2338 ms |
| claude-haiku-5-5 | markdown | 18 | 88.9% | 16 | 0 | 0 | 0 | 2 | 0 | 0 | 12683 | 1758 ms |
| claude-haiku-5-5 | toon | 18 | 88.9% | 16 | 0 | 0 | 0 | 2 | 0 | 0 | 16245 | 1749 ms |
| claude-haiku-5-5 | toon-tab | 18 | 83.3% | 15 | 0 | 0 | 0 | 3 | 0 | 0 | 16249 | 1956 ms |
| claude-haiku-5-5 | tsv | 18 | 77.8% | 14 | 1 | 1 | 0 | 2 | 0 | 0 | 7971 | 2094 ms |
| claude-haiku-5-5 | yaml | 18 | 88.9% | 16 | 0 | 0 | 0 | 2 | 0 | 0 | 15717 | 1734 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | csv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | csv | filter | 3 | 66.7% |
| claude-haiku-5-5 | csv | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | csv | compare | 3 | 100.0% |
| claude-haiku-5-5 | csv | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | csv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json | lookup | 3 | 100.0% |
| claude-haiku-5-5 | json | filter | 3 | 100.0% |
| claude-haiku-5-5 | json | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | json | compare | 3 | 100.0% |
| claude-haiku-5-5 | json | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | lookup | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | filter | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | lookup | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | filter | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | jsonl | compare | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | ldf | lookup | 3 | 100.0% |
| claude-haiku-5-5 | ldf | filter | 3 | 33.3% |
| claude-haiku-5-5 | ldf | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | ldf | compare | 3 | 66.7% |
| claude-haiku-5-5 | ldf | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | ldf | multi_record | 3 | 66.7% |
| claude-haiku-5-5 | markdown | lookup | 3 | 100.0% |
| claude-haiku-5-5 | markdown | filter | 3 | 100.0% |
| claude-haiku-5-5 | markdown | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | markdown | compare | 3 | 100.0% |
| claude-haiku-5-5 | markdown | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | markdown | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon | lookup | 3 | 100.0% |
| claude-haiku-5-5 | toon | filter | 3 | 100.0% |
| claude-haiku-5-5 | toon | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | toon | compare | 3 | 100.0% |
| claude-haiku-5-5 | toon | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | toon | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | lookup | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | filter | 3 | 66.7% |
| claude-haiku-5-5 | toon-tab | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | tsv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | tsv | filter | 3 | 66.7% |
| claude-haiku-5-5 | tsv | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | tsv | compare | 3 | 66.7% |
| claude-haiku-5-5 | tsv | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | tsv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | yaml | lookup | 3 | 100.0% |
| claude-haiku-5-5 | yaml | filter | 3 | 100.0% |
| claude-haiku-5-5 | yaml | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | yaml | compare | 3 | 100.0% |
| claude-haiku-5-5 | yaml | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | yaml | multi_record | 3 | 100.0% |
