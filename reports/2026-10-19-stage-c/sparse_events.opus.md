# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 80 | 97.5% | 78 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 15070 | 2247 ms |
| claude-opus-5-5 | ldf | 80 | 97.5% | 78 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 11521 | 2652 ms |
| claude-opus-5-5 | ldf-s75 | 80 | 95.0% | 76 | 3 | 1 | 0 | 0 | 0 | 0 | 0 | 12891 | 2315 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 10 | 100.0% |
| claude-opus-5-5 | json | filter | 10 | 100.0% |
| claude-opus-5-5 | json | aggregate | 10 | 80.0% |
| claude-opus-5-5 | json | compare | 10 | 100.0% |
| claude-opus-5-5 | json | missing_value | 10 | 100.0% |
| claude-opus-5-5 | json | multi_record | 10 | 100.0% |
| claude-opus-5-5 | json | aggregate_where | 10 | 100.0% |
| claude-opus-5-5 | json | rank | 10 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 10 | 100.0% |
| claude-opus-5-5 | ldf | filter | 10 | 100.0% |
| claude-opus-5-5 | ldf | aggregate | 10 | 100.0% |
| claude-opus-5-5 | ldf | compare | 10 | 100.0% |
| claude-opus-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 10 | 100.0% |
| claude-opus-5-5 | ldf | aggregate_where | 10 | 90.0% |
| claude-opus-5-5 | ldf | rank | 10 | 90.0% |
| claude-opus-5-5 | ldf-s75 | lookup | 10 | 100.0% |
| claude-opus-5-5 | ldf-s75 | filter | 10 | 100.0% |
| claude-opus-5-5 | ldf-s75 | aggregate | 10 | 90.0% |
| claude-opus-5-5 | ldf-s75 | compare | 10 | 100.0% |
| claude-opus-5-5 | ldf-s75 | missing_value | 10 | 100.0% |
| claude-opus-5-5 | ldf-s75 | multi_record | 10 | 80.0% |
| claude-opus-5-5 | ldf-s75 | aggregate_where | 10 | 100.0% |
| claude-opus-5-5 | ldf-s75 | rank | 10 | 90.0% |
