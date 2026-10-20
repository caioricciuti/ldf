# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | csv | 15 | 66.7% | 10 | 5 | 0 | 0 | 0 | 0 | 0 | 6159 | 948 ms |
| deepseek-chat | json | 15 | 73.3% | 11 | 4 | 0 | 0 | 0 | 0 | 0 | 10086 | 1169 ms |
| deepseek-chat | json-pretty | 15 | 73.3% | 11 | 4 | 0 | 0 | 0 | 0 | 0 | 15289 | 1364 ms |
| deepseek-chat | jsonl | 15 | 66.7% | 10 | 5 | 0 | 0 | 0 | 0 | 0 | 10086 | 1139 ms |
| deepseek-chat | ldf | 15 | 80.0% | 12 | 3 | 0 | 0 | 0 | 0 | 0 | 6174 | 1038 ms |
| deepseek-chat | markdown | 15 | 73.3% | 11 | 4 | 0 | 0 | 0 | 0 | 0 | 7255 | 1234 ms |
| deepseek-chat | toon | 15 | 80.0% | 12 | 3 | 0 | 0 | 0 | 0 | 0 | 6364 | 1044 ms |
| deepseek-chat | toon-tab | 15 | 80.0% | 12 | 3 | 0 | 0 | 0 | 0 | 0 | 6430 | 1039 ms |
| deepseek-chat | tsv | 15 | 60.0% | 9 | 6 | 0 | 0 | 0 | 0 | 0 | 6223 | 1064 ms |
| deepseek-chat | yaml | 15 | 66.7% | 10 | 5 | 0 | 0 | 0 | 0 | 0 | 12231 | 1135 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | csv | lookup | 3 | 100.0% |
| deepseek-chat | csv | filter | 3 | 33.3% |
| deepseek-chat | csv | aggregate | 3 | 33.3% |
| deepseek-chat | csv | compare | 3 | 100.0% |
| deepseek-chat | csv | multi_record | 3 | 66.7% |
| deepseek-chat | json | lookup | 3 | 100.0% |
| deepseek-chat | json | filter | 3 | 100.0% |
| deepseek-chat | json | aggregate | 3 | 33.3% |
| deepseek-chat | json | compare | 3 | 100.0% |
| deepseek-chat | json | multi_record | 3 | 33.3% |
| deepseek-chat | json-pretty | lookup | 3 | 100.0% |
| deepseek-chat | json-pretty | filter | 3 | 33.3% |
| deepseek-chat | json-pretty | aggregate | 3 | 66.7% |
| deepseek-chat | json-pretty | compare | 3 | 100.0% |
| deepseek-chat | json-pretty | multi_record | 3 | 66.7% |
| deepseek-chat | jsonl | lookup | 3 | 100.0% |
| deepseek-chat | jsonl | filter | 3 | 33.3% |
| deepseek-chat | jsonl | aggregate | 3 | 33.3% |
| deepseek-chat | jsonl | compare | 3 | 100.0% |
| deepseek-chat | jsonl | multi_record | 3 | 66.7% |
| deepseek-chat | ldf | lookup | 3 | 100.0% |
| deepseek-chat | ldf | filter | 3 | 66.7% |
| deepseek-chat | ldf | aggregate | 3 | 66.7% |
| deepseek-chat | ldf | compare | 3 | 100.0% |
| deepseek-chat | ldf | multi_record | 3 | 66.7% |
| deepseek-chat | markdown | lookup | 3 | 100.0% |
| deepseek-chat | markdown | filter | 3 | 33.3% |
| deepseek-chat | markdown | aggregate | 3 | 66.7% |
| deepseek-chat | markdown | compare | 3 | 100.0% |
| deepseek-chat | markdown | multi_record | 3 | 66.7% |
| deepseek-chat | toon | lookup | 3 | 100.0% |
| deepseek-chat | toon | filter | 3 | 66.7% |
| deepseek-chat | toon | aggregate | 3 | 66.7% |
| deepseek-chat | toon | compare | 3 | 100.0% |
| deepseek-chat | toon | multi_record | 3 | 66.7% |
| deepseek-chat | toon-tab | lookup | 3 | 100.0% |
| deepseek-chat | toon-tab | filter | 3 | 66.7% |
| deepseek-chat | toon-tab | aggregate | 3 | 66.7% |
| deepseek-chat | toon-tab | compare | 3 | 100.0% |
| deepseek-chat | toon-tab | multi_record | 3 | 66.7% |
| deepseek-chat | tsv | lookup | 3 | 100.0% |
| deepseek-chat | tsv | filter | 3 | 0.0% |
| deepseek-chat | tsv | aggregate | 3 | 66.7% |
| deepseek-chat | tsv | compare | 3 | 100.0% |
| deepseek-chat | tsv | multi_record | 3 | 33.3% |
| deepseek-chat | yaml | lookup | 3 | 100.0% |
| deepseek-chat | yaml | filter | 3 | 0.0% |
| deepseek-chat | yaml | aggregate | 3 | 33.3% |
| deepseek-chat | yaml | compare | 3 | 100.0% |
| deepseek-chat | yaml | multi_record | 3 | 100.0% |
