# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 60 | 98.3% | 59 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 75879 | 2667 ms |
| claude-sonnet-5-5 | ldf | 60 | 90.0% | 54 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 58108 | 2742 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | json | filter | 10 | 90.0% |
| claude-sonnet-5-5 | json | compare | 10 | 100.0% |
| claude-sonnet-5-5 | json | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | json | rank | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 10 | 50.0% |
| claude-sonnet-5-5 | ldf | compare | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | multi_record | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | rank | 10 | 90.0% |
