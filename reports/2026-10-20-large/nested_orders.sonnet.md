# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 68 | 95.6% | 65 | 1 | 0 | 2 | 0 | 0 | 0 | 0 | 151189 | 2610 ms |
| claude-sonnet-5-5 | ldf | 68 | 83.8% | 57 | 3 | 0 | 8 | 0 | 0 | 0 | 0 | 86176 | 2354 ms |
| claude-sonnet-5-5 | ldf-s75 | 68 | 85.3% | 58 | 2 | 0 | 8 | 0 | 0 | 0 | 0 | 92620 | 2668 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | json | filter | 8 | 100.0% |
| claude-sonnet-5-5 | json | compare | 10 | 70.0% |
| claude-sonnet-5-5 | json | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | json | nested_extract | 10 | 100.0% |
| claude-sonnet-5-5 | json | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | json | child_count | 10 | 100.0% |
| claude-sonnet-5-5 | json | rank | 8 | 100.0% |
| claude-sonnet-5-5 | ldf | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 8 | 100.0% |
| claude-sonnet-5-5 | ldf | compare | 10 | 20.0% |
| claude-sonnet-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | nested_extract | 10 | 90.0% |
| claude-sonnet-5-5 | ldf | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf | child_count | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | rank | 8 | 75.0% |
| claude-sonnet-5-5 | ldf-s75 | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | filter | 8 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | compare | 10 | 10.0% |
| claude-sonnet-5-5 | ldf-s75 | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | nested_extract | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | child_count | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | rank | 8 | 87.5% |
