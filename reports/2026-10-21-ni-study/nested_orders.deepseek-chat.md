# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | json | 249 | 65.1% | 162 | 87 | 0 | 0 | 0 | 0 | 0 | 0 | 21626 | 1269 ms |
| deepseek-chat | ldf-hoisted | 249 | 63.9% | 159 | 87 | 0 | 0 | 0 | 3 | 0 | 0 | 11927 | 1406 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | json | lookup | 30 | 96.7% |
| deepseek-chat | json | filter | 24 | 8.3% |
| deepseek-chat | json | aggregate | 5 | 60.0% |
| deepseek-chat | json | compare | 30 | 90.0% |
| deepseek-chat | json | missing_value | 30 | 100.0% |
| deepseek-chat | json | nested_extract | 30 | 96.7% |
| deepseek-chat | json | multi_record | 2 | 100.0% |
| deepseek-chat | json | child_count | 30 | 100.0% |
| deepseek-chat | json | child_aggregate | 30 | 3.3% |
| deepseek-chat | json | aggregate_where | 30 | 10.0% |
| deepseek-chat | json | rank | 8 | 75.0% |
| deepseek-chat | ldf-hoisted | lookup | 30 | 100.0% |
| deepseek-chat | ldf-hoisted | filter | 24 | 8.3% |
| deepseek-chat | ldf-hoisted | aggregate | 5 | 60.0% |
| deepseek-chat | ldf-hoisted | compare | 30 | 96.7% |
| deepseek-chat | ldf-hoisted | missing_value | 30 | 100.0% |
| deepseek-chat | ldf-hoisted | nested_extract | 30 | 100.0% |
| deepseek-chat | ldf-hoisted | multi_record | 2 | 100.0% |
| deepseek-chat | ldf-hoisted | child_count | 30 | 83.3% |
| deepseek-chat | ldf-hoisted | child_aggregate | 30 | 0.0% |
| deepseek-chat | ldf-hoisted | aggregate_where | 30 | 6.7% |
| deepseek-chat | ldf-hoisted | rank | 8 | 75.0% |
