# LLM comprehension results

| model | format | n | accuracy | correct | wrong | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | csv | 15 | 73.3% | 11 | 0 | 0 | 4 | 0 | 0 | 19147 | 2446 ms |
| claude-haiku-5-5 | json | 15 | 66.7% | 10 | 1 | 0 | 4 | 0 | 0 | 34136 | 2195 ms |
| claude-haiku-5-5 | json-pretty | 15 | 73.3% | 11 | 1 | 0 | 3 | 0 | 0 | 47141 | 2143 ms |
| claude-haiku-5-5 | jsonl | 15 | 73.3% | 11 | 2 | 0 | 2 | 0 | 0 | 34636 | 2017 ms |
| claude-haiku-5-5 | ldf | 15 | 66.7% | 10 | 2 | 0 | 3 | 0 | 0 | 19172 | 2230 ms |
| claude-haiku-5-5 | markdown | 15 | 73.3% | 11 | 1 | 0 | 3 | 0 | 0 | 24176 | 2174 ms |
| claude-haiku-5-5 | toon | 15 | 60.0% | 9 | 3 | 0 | 3 | 0 | 0 | 19653 | 2298 ms |
| claude-haiku-5-5 | toon-tab | 15 | 73.3% | 11 | 1 | 0 | 3 | 0 | 0 | 19657 | 2354 ms |
| claude-haiku-5-5 | tsv | 15 | 86.7% | 13 | 0 | 0 | 2 | 0 | 0 | 19148 | 2035 ms |
| claude-haiku-5-5 | yaml | 15 | 80.0% | 12 | 1 | 0 | 2 | 0 | 0 | 34683 | 1879 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | csv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | csv | filter | 3 | 0.0% |
| claude-haiku-5-5 | csv | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | csv | compare | 3 | 100.0% |
| claude-haiku-5-5 | csv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json | lookup | 3 | 66.7% |
| claude-haiku-5-5 | json | filter | 3 | 0.0% |
| claude-haiku-5-5 | json | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | json | compare | 3 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | lookup | 3 | 66.7% |
| claude-haiku-5-5 | json-pretty | filter | 3 | 33.3% |
| claude-haiku-5-5 | json-pretty | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-haiku-5-5 | json-pretty | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | lookup | 3 | 66.7% |
| claude-haiku-5-5 | jsonl | filter | 3 | 33.3% |
| claude-haiku-5-5 | jsonl | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | jsonl | compare | 3 | 100.0% |
| claude-haiku-5-5 | jsonl | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | ldf | lookup | 3 | 66.7% |
| claude-haiku-5-5 | ldf | filter | 3 | 33.3% |
| claude-haiku-5-5 | ldf | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | ldf | compare | 3 | 66.7% |
| claude-haiku-5-5 | ldf | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | markdown | lookup | 3 | 100.0% |
| claude-haiku-5-5 | markdown | filter | 3 | 33.3% |
| claude-haiku-5-5 | markdown | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | markdown | compare | 3 | 66.7% |
| claude-haiku-5-5 | markdown | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon | lookup | 3 | 66.7% |
| claude-haiku-5-5 | toon | filter | 3 | 0.0% |
| claude-haiku-5-5 | toon | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | toon | compare | 3 | 66.7% |
| claude-haiku-5-5 | toon | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | lookup | 3 | 66.7% |
| claude-haiku-5-5 | toon-tab | filter | 3 | 33.3% |
| claude-haiku-5-5 | toon-tab | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-haiku-5-5 | toon-tab | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | tsv | lookup | 3 | 100.0% |
| claude-haiku-5-5 | tsv | filter | 3 | 66.7% |
| claude-haiku-5-5 | tsv | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | tsv | compare | 3 | 100.0% |
| claude-haiku-5-5 | tsv | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | yaml | lookup | 3 | 100.0% |
| claude-haiku-5-5 | yaml | filter | 3 | 33.3% |
| claude-haiku-5-5 | yaml | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | yaml | compare | 3 | 100.0% |
| claude-haiku-5-5 | yaml | multi_record | 3 | 100.0% |
