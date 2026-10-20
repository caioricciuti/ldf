# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 60 | 93.3% | 56 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 75879 | 2987 ms |
| claude-opus-5-5 | ldf | 60 | 95.0% | 57 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 58108 | 3164 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 10 | 100.0% |
| claude-opus-5-5 | json | filter | 10 | 80.0% |
| claude-opus-5-5 | json | compare | 10 | 80.0% |
| claude-opus-5-5 | json | missing_value | 10 | 100.0% |
| claude-opus-5-5 | json | multi_record | 10 | 100.0% |
| claude-opus-5-5 | json | rank | 10 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 10 | 100.0% |
| claude-opus-5-5 | ldf | filter | 10 | 90.0% |
| claude-opus-5-5 | ldf | compare | 10 | 100.0% |
| claude-opus-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 10 | 80.0% |
| claude-opus-5-5 | ldf | rank | 10 | 100.0% |
