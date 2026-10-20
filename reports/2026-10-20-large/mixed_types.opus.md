# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 52 | 100.0% | 52 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 38761 | 2101 ms |
| claude-opus-5-5 | ldf | 52 | 100.0% | 52 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 29399 | 2194 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 10 | 100.0% |
| claude-opus-5-5 | json | filter | 10 | 100.0% |
| claude-opus-5-5 | json | compare | 10 | 100.0% |
| claude-opus-5-5 | json | missing_value | 10 | 100.0% |
| claude-opus-5-5 | json | nested_extract | 10 | 100.0% |
| claude-opus-5-5 | json | multi_record | 2 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 10 | 100.0% |
| claude-opus-5-5 | ldf | filter | 10 | 100.0% |
| claude-opus-5-5 | ldf | compare | 10 | 100.0% |
| claude-opus-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-opus-5-5 | ldf | nested_extract | 10 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 2 | 100.0% |
