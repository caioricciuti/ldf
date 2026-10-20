# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 80 | 97.5% | 78 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 15070 | 2401 ms |
| claude-sonnet-5-5 | ldf | 80 | 96.2% | 77 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 11521 | 2274 ms |
| claude-sonnet-5-5 | ldf-s75 | 80 | 93.8% | 75 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 12891 | 2415 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | json | filter | 10 | 90.0% |
| claude-sonnet-5-5 | json | aggregate | 10 | 100.0% |
| claude-sonnet-5-5 | json | compare | 10 | 100.0% |
| claude-sonnet-5-5 | json | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | json | aggregate_where | 10 | 90.0% |
| claude-sonnet-5-5 | json | rank | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate | 10 | 80.0% |
| claude-sonnet-5-5 | ldf | compare | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate_where | 10 | 90.0% |
| claude-sonnet-5-5 | ldf | rank | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | filter | 10 | 90.0% |
| claude-sonnet-5-5 | ldf-s75 | aggregate | 10 | 80.0% |
| claude-sonnet-5-5 | ldf-s75 | compare | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | aggregate_where | 10 | 90.0% |
| claude-sonnet-5-5 | ldf-s75 | rank | 10 | 90.0% |
