# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 151 | 100.0% | 151 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8435 | 1421 ms |
| claude-sonnet-5-5 | ldf | 151 | 100.0% | 151 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 6593 | 1454 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | json | filter | 24 | 100.0% |
| claude-sonnet-5-5 | json | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | json | compare | 30 | 100.0% |
| claude-sonnet-5-5 | json | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | json | nested_extract | 30 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 24 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | ldf | compare | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | nested_extract | 30 | 100.0% |
| claude-sonnet-5-5 | ldf | multi_record | 2 | 100.0% |
