# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 220 | 98.2% | 216 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 15653 | 2256 ms |
| claude-sonnet-5-5 | ldf | 220 | 97.3% | 214 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 12104 | 2284 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | json | filter | 30 | 96.7% |
| claude-sonnet-5-5 | json | aggregate | 30 | 93.3% |
| claude-sonnet-5-5 | json | compare | 30 | 100.0% |
| claude-sonnet-5-5 | json | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | json | aggregate_where | 30 | 100.0% |
| claude-sonnet-5-5 | json | rank | 30 | 96.7% |
| claude-sonnet-5-5 | ldf | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 30 | 86.7% |
| claude-sonnet-5-5 | ldf | aggregate | 30 | 93.3% |
| claude-sonnet-5-5 | ldf | compare | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate_where | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | rank | 30 | 100.0% |
