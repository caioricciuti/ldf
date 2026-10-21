# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 220 | 97.3% | 214 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 15653 | 2365 ms |
| claude-opus-5-5 | ldf | 220 | 97.3% | 214 | 4 | 2 | 0 | 0 | 0 | 0 | 0 | 12104 | 2705 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 30 | 100.0% |
| claude-opus-5-5 | json | filter | 30 | 90.0% |
| claude-opus-5-5 | json | aggregate | 30 | 96.7% |
| claude-opus-5-5 | json | compare | 30 | 96.7% |
| claude-opus-5-5 | json | missing_value | 30 | 100.0% |
| claude-opus-5-5 | json | multi_record | 10 | 100.0% |
| claude-opus-5-5 | json | aggregate_where | 30 | 96.7% |
| claude-opus-5-5 | json | rank | 30 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 30 | 100.0% |
| claude-opus-5-5 | ldf | filter | 30 | 90.0% |
| claude-opus-5-5 | ldf | aggregate | 30 | 100.0% |
| claude-opus-5-5 | ldf | compare | 30 | 93.3% |
| claude-opus-5-5 | ldf | missing_value | 30 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 10 | 100.0% |
| claude-opus-5-5 | ldf | aggregate_where | 30 | 100.0% |
| claude-opus-5-5 | ldf | rank | 30 | 96.7% |
