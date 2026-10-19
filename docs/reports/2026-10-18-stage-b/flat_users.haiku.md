# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | csv | 89 | 96.6% | 86 | 3 | 0 | 0 | 0 | 0 | 0 | 7755 | 2166 ms |
| claude-haiku-5-5 | json | 89 | 97.8% | 87 | 2 | 0 | 0 | 0 | 0 | 0 | 13742 | 2043 ms |
| claude-haiku-5-5 | ldf | 89 | 97.8% | 87 | 2 | 0 | 0 | 0 | 0 | 0 | 7778 | 2247 ms |
| claude-haiku-5-5 | ldf-global | 89 | 100.0% | 89 | 0 | 0 | 0 | 0 | 0 | 0 | 7762 | 2344 ms |
| claude-haiku-5-5 | ldf-tab | 89 | 97.8% | 87 | 2 | 0 | 0 | 0 | 0 | 0 | 7778 | 2168 ms |
| claude-haiku-5-5 | toon | 89 | 96.6% | 86 | 3 | 0 | 0 | 0 | 0 | 0 | 7961 | 2268 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | csv | lookup | 15 | 86.7% |
| claude-haiku-5-5 | csv | filter | 15 | 100.0% |
| claude-haiku-5-5 | csv | aggregate | 15 | 100.0% |
| claude-haiku-5-5 | csv | compare | 15 | 100.0% |
| claude-haiku-5-5 | csv | multi_record | 5 | 100.0% |
| claude-haiku-5-5 | csv | aggregate_where | 15 | 93.3% |
| claude-haiku-5-5 | csv | rank | 9 | 100.0% |
| claude-haiku-5-5 | json | lookup | 15 | 100.0% |
| claude-haiku-5-5 | json | filter | 15 | 93.3% |
| claude-haiku-5-5 | json | aggregate | 15 | 100.0% |
| claude-haiku-5-5 | json | compare | 15 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 5 | 100.0% |
| claude-haiku-5-5 | json | aggregate_where | 15 | 93.3% |
| claude-haiku-5-5 | json | rank | 9 | 100.0% |
| claude-haiku-5-5 | ldf | lookup | 15 | 93.3% |
| claude-haiku-5-5 | ldf | filter | 15 | 93.3% |
| claude-haiku-5-5 | ldf | aggregate | 15 | 100.0% |
| claude-haiku-5-5 | ldf | compare | 15 | 100.0% |
| claude-haiku-5-5 | ldf | multi_record | 5 | 100.0% |
| claude-haiku-5-5 | ldf | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | ldf | rank | 9 | 100.0% |
| claude-haiku-5-5 | ldf-global | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | aggregate | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | compare | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | multi_record | 5 | 100.0% |
| claude-haiku-5-5 | ldf-global | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | rank | 9 | 100.0% |
| claude-haiku-5-5 | ldf-tab | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | filter | 15 | 86.7% |
| claude-haiku-5-5 | ldf-tab | aggregate | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | compare | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | multi_record | 5 | 100.0% |
| claude-haiku-5-5 | ldf-tab | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | rank | 9 | 100.0% |
| claude-haiku-5-5 | toon | lookup | 15 | 93.3% |
| claude-haiku-5-5 | toon | filter | 15 | 100.0% |
| claude-haiku-5-5 | toon | aggregate | 15 | 100.0% |
| claude-haiku-5-5 | toon | compare | 15 | 93.3% |
| claude-haiku-5-5 | toon | multi_record | 5 | 100.0% |
| claude-haiku-5-5 | toon | aggregate_where | 15 | 93.3% |
| claude-haiku-5-5 | toon | rank | 9 | 100.0% |
