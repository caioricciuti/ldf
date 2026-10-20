# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | ldf-s25 | 18 | 88.9% | 16 | 0 | 1 | 0 | 1 | 0 | 0 | 10290 | 3146 ms |
| claude-haiku-5-5 | ldf-s50 | 18 | 94.4% | 17 | 0 | 1 | 0 | 0 | 0 | 0 | 11443 | 2178 ms |
| claude-haiku-5-5 | ldf-s75 | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 13138 | 2293 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | ldf-s25 | lookup | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s25 | filter | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s25 | aggregate | 3 | 66.7% |
| claude-haiku-5-5 | ldf-s25 | compare | 3 | 66.7% |
| claude-haiku-5-5 | ldf-s25 | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s25 | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s50 | lookup | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s50 | filter | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s50 | aggregate | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s50 | compare | 3 | 66.7% |
| claude-haiku-5-5 | ldf-s50 | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s50 | multi_record | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s75 | lookup | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s75 | filter | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s75 | aggregate | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s75 | compare | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s75 | missing_value | 3 | 100.0% |
| claude-haiku-5-5 | ldf-s75 | multi_record | 3 | 100.0% |
