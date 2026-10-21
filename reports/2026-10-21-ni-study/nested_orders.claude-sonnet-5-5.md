# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 249 | 92.0% | 229 | 20 | 0 | 0 | 0 | 0 | 0 | 0 | 30873 | 2672 ms |
| claude-sonnet-5-5 | ldf-s75 | 249 | 91.2% | 227 | 21 | 0 | 1 | 0 | 0 | 0 | 0 | 19264 | 2673 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-sonnet-5-5 | json | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | json | filter | 24 | 91.7% |
| claude-sonnet-5-5 | json | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | json | compare | 30 | 100.0% |
| claude-sonnet-5-5 | json | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | json | nested_extract | 30 | 96.7% |
| claude-sonnet-5-5 | json | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | json | child_count | 30 | 100.0% |
| claude-sonnet-5-5 | json | child_aggregate | 30 | 56.7% |
| claude-sonnet-5-5 | json | aggregate_where | 30 | 86.7% |
| claude-sonnet-5-5 | json | rank | 8 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | lookup | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | filter | 24 | 95.8% |
| claude-sonnet-5-5 | ldf-s75 | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | compare | 30 | 96.7% |
| claude-sonnet-5-5 | ldf-s75 | missing_value | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | nested_extract | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | child_count | 30 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | child_aggregate | 30 | 43.3% |
| claude-sonnet-5-5 | ldf-s75 | aggregate_where | 30 | 90.0% |
| claude-sonnet-5-5 | ldf-s75 | rank | 8 | 100.0% |
