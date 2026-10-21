# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 249 | 99.2% | 247 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 30873 | 3146 ms |
| claude-opus-5-5 | ldf-hoisted | 249 | 95.6% | 238 | 10 | 1 | 0 | 0 | 0 | 0 | 0 | 14951 | 3027 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 30 | 100.0% |
| claude-opus-5-5 | json | filter | 24 | 100.0% |
| claude-opus-5-5 | json | aggregate | 5 | 100.0% |
| claude-opus-5-5 | json | compare | 30 | 100.0% |
| claude-opus-5-5 | json | missing_value | 30 | 100.0% |
| claude-opus-5-5 | json | nested_extract | 30 | 100.0% |
| claude-opus-5-5 | json | multi_record | 2 | 100.0% |
| claude-opus-5-5 | json | child_count | 30 | 100.0% |
| claude-opus-5-5 | json | child_aggregate | 30 | 100.0% |
| claude-opus-5-5 | json | aggregate_where | 30 | 93.3% |
| claude-opus-5-5 | json | rank | 8 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | lookup | 30 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | filter | 24 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | aggregate | 5 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | compare | 30 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | missing_value | 30 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | nested_extract | 30 | 90.0% |
| claude-opus-5-5 | ldf-hoisted | multi_record | 2 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | child_count | 30 | 100.0% |
| claude-opus-5-5 | ldf-hoisted | child_aggregate | 30 | 83.3% |
| claude-opus-5-5 | ldf-hoisted | aggregate_where | 30 | 90.0% |
| claude-opus-5-5 | ldf-hoisted | rank | 8 | 100.0% |
