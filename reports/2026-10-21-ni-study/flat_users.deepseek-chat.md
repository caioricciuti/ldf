# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | json | 151 | 60.9% | 92 | 59 | 0 | 0 | 0 | 0 | 0 | 0 | 10535 | 1127 ms |
| deepseek-chat | ldf | 151 | 55.0% | 83 | 68 | 0 | 0 | 0 | 0 | 0 | 0 | 6636 | 1134 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | json | lookup | 30 | 100.0% |
| deepseek-chat | json | filter | 30 | 53.3% |
| deepseek-chat | json | aggregate | 15 | 73.3% |
| deepseek-chat | json | compare | 30 | 76.7% |
| deepseek-chat | json | multi_record | 5 | 60.0% |
| deepseek-chat | json | aggregate_where | 30 | 16.7% |
| deepseek-chat | json | rank | 11 | 36.4% |
| deepseek-chat | ldf | lookup | 30 | 100.0% |
| deepseek-chat | ldf | filter | 30 | 36.7% |
| deepseek-chat | ldf | aggregate | 15 | 73.3% |
| deepseek-chat | ldf | compare | 30 | 73.3% |
| deepseek-chat | ldf | multi_record | 5 | 40.0% |
| deepseek-chat | ldf | aggregate_where | 30 | 10.0% |
| deepseek-chat | ldf | rank | 11 | 36.4% |
