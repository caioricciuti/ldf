# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 65 | 100.0% | 65 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 13723 | 2666 ms |
| claude-opus-5-5 | ldf | 65 | 100.0% | 65 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7759 | 2566 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 10 | 100.0% |
| claude-opus-5-5 | json | filter | 10 | 100.0% |
| claude-opus-5-5 | json | aggregate | 10 | 100.0% |
| claude-opus-5-5 | json | compare | 10 | 100.0% |
| claude-opus-5-5 | json | multi_record | 5 | 100.0% |
| claude-opus-5-5 | json | aggregate_where | 10 | 100.0% |
| claude-opus-5-5 | json | rank | 10 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 10 | 100.0% |
| claude-opus-5-5 | ldf | filter | 10 | 100.0% |
| claude-opus-5-5 | ldf | aggregate | 10 | 100.0% |
| claude-opus-5-5 | ldf | compare | 10 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 5 | 100.0% |
| claude-opus-5-5 | ldf | aggregate_where | 10 | 100.0% |
| claude-opus-5-5 | ldf | rank | 10 | 100.0% |
