# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 57 | 100.0% | 57 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7851 | 1557 ms |
| claude-sonnet-5-5 | ldf | 57 | 100.0% | 57 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 6009 | 1637 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | json | filter | 10 | 100.0% |
| claude-sonnet-5-5 | json | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | json | compare | 10 | 100.0% |
| claude-sonnet-5-5 | json | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | json | nested_extract | 10 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | ldf | compare | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | nested_extract | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | multi_record | 2 | 100.0% |
