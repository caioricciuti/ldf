# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 151 | 99.3% | 150 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 14307 | 2083 ms |
| claude-sonnet-5-5 | ldf | 151 | 98.0% | 148 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 8343 | 2287 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | json | filter | 30 | 100.0% |
| claude-sonnet-5-5 | json | aggregate | 15 | 100.0% |
| claude-sonnet-5-5 | json | compare | 30 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 5 | 100.0% |
| claude-sonnet-5-5 | json | aggregate_where | 30 | 96.7% |
| claude-sonnet-5-5 | json | rank | 11 | 100.0% |
| claude-sonnet-5-5 | ldf | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate | 15 | 100.0% |
| claude-sonnet-5-5 | ldf | compare | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | multi_record | 5 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate_where | 30 | 90.0% |
| claude-sonnet-5-5 | ldf | rank | 11 | 100.0% |
