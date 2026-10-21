# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 220 | 98.6% | 217 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 24098 | 2297 ms |
| claude-sonnet-5-5 | ldf-s0 | 220 | 98.2% | 216 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 11523 | 2331 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | json | filter | 30 | 100.0% |
| claude-sonnet-5-5 | json | aggregate | 30 | 93.3% |
| claude-sonnet-5-5 | json | compare | 30 | 100.0% |
| claude-sonnet-5-5 | json | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | json | aggregate_where | 30 | 96.7% |
| claude-sonnet-5-5 | json | rank | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s0 | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s0 | filter | 30 | 96.7% |
| claude-sonnet-5-5 | ldf-s0 | aggregate | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s0 | compare | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s0 | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s0 | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s0 | aggregate_where | 30 | 93.3% |
| claude-sonnet-5-5 | ldf-s0 | rank | 30 | 96.7% |
