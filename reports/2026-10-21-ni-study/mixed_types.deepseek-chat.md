# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | json | 151 | 99.3% | 150 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 5782 | 1022 ms |
| deepseek-chat | ldf | 151 | 99.3% | 150 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 4460 | 1027 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | json | lookup | 30 | 100.0% |
| deepseek-chat | json | filter | 24 | 100.0% |
| deepseek-chat | json | aggregate | 5 | 100.0% |
| deepseek-chat | json | compare | 30 | 100.0% |
| deepseek-chat | json | missing_value | 30 | 100.0% |
| deepseek-chat | json | nested_extract | 30 | 96.7% |
| deepseek-chat | json | multi_record | 2 | 100.0% |
| deepseek-chat | ldf | lookup | 30 | 100.0% |
| deepseek-chat | ldf | filter | 24 | 100.0% |
| deepseek-chat | ldf | aggregate | 5 | 100.0% |
| deepseek-chat | ldf | compare | 30 | 100.0% |
| deepseek-chat | ldf | missing_value | 30 | 96.7% |
| deepseek-chat | ldf | nested_extract | 30 | 100.0% |
| deepseek-chat | ldf | multi_record | 2 | 100.0% |
