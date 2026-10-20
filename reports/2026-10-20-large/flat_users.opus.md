# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 64 | 100.0% | 64 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 68089 | 2573 ms |
| claude-opus-5-5 | ldf | 64 | 92.2% | 59 | 2 | 3 | 0 | 0 | 0 | 0 | 0 | 38125 | 2741 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 20 | 100.0% |
| claude-opus-5-5 | json | filter | 14 | 100.0% |
| claude-opus-5-5 | json | compare | 20 | 100.0% |
| claude-opus-5-5 | json | multi_record | 4 | 100.0% |
| claude-opus-5-5 | json | rank | 6 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 20 | 100.0% |
| claude-opus-5-5 | ldf | filter | 14 | 92.9% |
| claude-opus-5-5 | ldf | compare | 20 | 80.0% |
| claude-opus-5-5 | ldf | multi_record | 4 | 100.0% |
| claude-opus-5-5 | ldf | rank | 6 | 100.0% |
