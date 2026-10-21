# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | json | 220 | 61.8% | 136 | 84 | 0 | 0 | 0 | 0 | 0 | 0 | 11678 | 1215 ms |
| deepseek-chat | ldf-s75 | 220 | 60.0% | 132 | 88 | 0 | 0 | 0 | 0 | 0 | 0 | 10383 | 1114 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | json | lookup | 30 | 100.0% |
| deepseek-chat | json | filter | 30 | 20.0% |
| deepseek-chat | json | aggregate | 30 | 63.3% |
| deepseek-chat | json | compare | 30 | 80.0% |
| deepseek-chat | json | missing_value | 30 | 93.3% |
| deepseek-chat | json | multi_record | 10 | 70.0% |
| deepseek-chat | json | aggregate_where | 30 | 16.7% |
| deepseek-chat | json | rank | 30 | 56.7% |
| deepseek-chat | ldf-s75 | lookup | 30 | 96.7% |
| deepseek-chat | ldf-s75 | filter | 30 | 30.0% |
| deepseek-chat | ldf-s75 | aggregate | 30 | 63.3% |
| deepseek-chat | ldf-s75 | compare | 30 | 80.0% |
| deepseek-chat | ldf-s75 | missing_value | 30 | 90.0% |
| deepseek-chat | ldf-s75 | multi_record | 10 | 50.0% |
| deepseek-chat | ldf-s75 | aggregate_where | 30 | 13.3% |
| deepseek-chat | ldf-s75 | rank | 30 | 50.0% |
