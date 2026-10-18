# LLM comprehension results

| model | format | n | accuracy | correct | wrong | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | csv | 18 | 72.2% | 13 | 2 | 0 | 3 | 0 | 0 | 22984 | 2493 ms |
| claude-haiku-5-5 | json | 18 | 83.3% | 15 | 1 | 0 | 2 | 0 | 0 | 37811 | 2232 ms |
| claude-haiku-5-5 | json-pretty | 18 | 77.8% | 14 | 0 | 0 | 4 | 0 | 0 | 51113 | 2399 ms |
| claude-haiku-5-5 | jsonl | 18 | 83.3% | 15 | 0 | 0 | 3 | 0 | 0 | 38311 | 2204 ms |
| claude-haiku-5-5 | ldf | 18 | 66.7% | 12 | 3 | 0 | 3 | 0 | 0 | 23640 | 2524 ms |
| claude-haiku-5-5 | markdown | 18 | 83.3% | 15 | 1 | 0 | 2 | 0 | 0 | 31220 | 2121 ms |
| claude-haiku-5-5 | toon | 18 | 83.3% | 15 | 0 | 0 | 3 | 0 | 0 | 40113 | 2086 ms |
| claude-haiku-5-5 | toon-tab | 18 | 77.8% | 14 | 0 | 0 | 4 | 0 | 0 | 40117 | 2215 ms |
| claude-haiku-5-5 | tsv | 18 | 72.2% | 13 | 1 | 0 | 4 | 0 | 0 | 19395 | 2484 ms |
| claude-haiku-5-5 | yaml | 18 | 77.8% | 14 | 0 | 0 | 4 | 0 | 0 | 38835 | 2202 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | csv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | csv | filter | 3 | 33.3% |
| claude-haiku-5-5 | csv | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | csv | compare | 3 | 66.7% |
| claude-haiku-5-5 | csv | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | csv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json | lookup | 3 | 100.0% |
| claude-haiku-5-5 | json | filter | 3 | 33.3% |
| claude-haiku-5-5 | json | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | json | compare | 3 | 100.0% |
| claude-haiku-5-5 | json | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | lookup | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | filter | 3 | 0.0% |
| claude-haiku-5-5 | json-pretty | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | lookup | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | filter | 3 | 33.3% |
| claude-haiku-5-5 | jsonl | aggregate | 3 | 66.7% |
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
| claude-haiku-5-5 | markdown | filter | 3 | 66.7% |
| claude-haiku-5-5 | markdown | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | markdown | compare | 3 | 100.0% |
| claude-haiku-5-5 | markdown | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | markdown | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon | lookup | 3 | 100.0% |
| claude-haiku-5-5 | toon | filter | 3 | 33.3% |
| claude-haiku-5-5 | toon | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | toon | compare | 3 | 100.0% |
| claude-haiku-5-5 | toon | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | toon | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | lookup | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | filter | 3 | 0.0% |
| claude-haiku-5-5 | toon-tab | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | tsv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | tsv | filter | 3 | 0.0% |
| claude-haiku-5-5 | tsv | aggregate | 3 | 33.3% |
| claude-haiku-5-5 | tsv | compare | 3 | 100.0% |
| claude-haiku-5-5 | tsv | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | tsv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | yaml | lookup | 3 | 100.0% |
| claude-haiku-5-5 | yaml | filter | 3 | 0.0% |
| claude-haiku-5-5 | yaml | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | yaml | compare | 3 | 100.0% |
| claude-haiku-5-5 | yaml | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | yaml | multi_record | 3 | 100.0% |
