# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-sonnet-5-5 | json | 95 | 91.6% | 87 | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 30289 | 2797 ms |
| claude-sonnet-5-5 | ldf | 95 | 84.2% | 80 | 7 | 0 | 8 | 0 | 0 | 0 | 0 | 17390 | 2731 ms |
| claude-sonnet-5-5 | ldf-hoisted | 95 | 85.3% | 81 | 13 | 0 | 1 | 0 | 0 | 0 | 0 | 14367 | 2996 ms |
| claude-sonnet-5-5 | ldf-s75 | 95 | 89.5% | 85 | 7 | 0 | 3 | 0 | 0 | 0 | 0 | 18680 | 2675 ms |

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
| claude-sonnet-5-5 | json | child_count | 10 | 100.0% |
| claude-sonnet-5-5 | json | child_aggregate | 10 | 30.0% |
| claude-sonnet-5-5 | json | aggregate_where | 10 | 90.0% |
| claude-sonnet-5-5 | json | rank | 8 | 100.0% |
| claude-sonnet-5-5 | ldf | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | filter | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | ldf | compare | 10 | 20.0% |
| claude-sonnet-5-5 | ldf | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | nested_extract | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf | child_count | 10 | 100.0% |
| claude-sonnet-5-5 | ldf | child_aggregate | 10 | 50.0% |
| claude-sonnet-5-5 | ldf | aggregate_where | 10 | 80.0% |
| claude-sonnet-5-5 | ldf | rank | 8 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | filter | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | compare | 10 | 90.0% |
| claude-sonnet-5-5 | ldf-hoisted | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | nested_extract | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | child_count | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-hoisted | child_aggregate | 10 | 0.0% |
| claude-sonnet-5-5 | ldf-hoisted | aggregate_where | 10 | 80.0% |
| claude-sonnet-5-5 | ldf-hoisted | rank | 8 | 87.5% |
| claude-sonnet-5-5 | ldf-s75 | lookup | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | filter | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | aggregate | 5 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | compare | 10 | 70.0% |
| claude-sonnet-5-5 | ldf-s75 | missing_value | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | nested_extract | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | multi_record | 2 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | child_count | 10 | 100.0% |
| claude-sonnet-5-5 | ldf-s75 | child_aggregate | 10 | 50.0% |
| claude-sonnet-5-5 | ldf-s75 | aggregate_where | 10 | 80.0% |
| claude-sonnet-5-5 | ldf-s75 | rank | 8 | 100.0% |
