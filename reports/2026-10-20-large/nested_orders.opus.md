# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 68 | 97.1% | 66 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 151189 | 3208 ms |
| claude-opus-5-5 | ldf | 68 | 97.1% | 66 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 86176 | 3036 ms |
| claude-opus-5-5 | ldf-hoisted | 68 | 98.5% | 67 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 70961 | 3008 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 10 | 100.0% |
| claude-opus-5-5 | json | filter | 8 | 100.0% |
| claude-opus-5-5 | json | compare | 10 | 100.0% |
| claude-opus-5-5 | json | missing_value | 10 | 100.0% |
| claude-opus-5-5 | json | nested_extract | 10 | 100.0% |
| claude-opus-5-5 | json | multi_record | 2 | 100.0% |
| claude-opus-5-5 | json | child_count | 10 | 80.0% |
| claude-opus-5-5 | json | rank | 8 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 10 | 100.0% |
| claude-opus-5-5 | ldf | filter | 8 | 100.0% |
| claude-opus-5-5 | ldf | compare | 10 | 90.0% |
| claude-opus-5-5 | ldf | missing_value | 10 | 90.0% |
| claude-opus-5-5 | ldf | nested_extract | 10 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 2 | 100.0% |
| claude-opus-5-5 | ldf | child_count | 10 | 100.0% |
| claude-opus-5-5 | ldf | rank | 8 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | lookup | 10 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | filter | 8 | 87.5% |
| claude-opus-5-5 | ldf-hoisted | compare | 10 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | missing_value | 10 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | nested_extract | 10 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | multi_record | 2 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | child_count | 10 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | rank | 8 | 100.0% |
