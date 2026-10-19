# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | json | 135 | 96.3% | 130 | 4 | 0 | 0 | 1 | 0 | 0 | 30139 | 3402 ms |
| claude-haiku-5-5 | ldf | 135 | 91.9% | 124 | 10 | 0 | 0 | 1 | 0 | 0 | 17264 | 3056 ms |
| claude-haiku-5-5 | ldf-global | 135 | 75.6% | 102 | 33 | 0 | 0 | 0 | 0 | 0 | 14435 | 3058 ms |
| claude-haiku-5-5 | ldf-hoisted | 135 | 88.9% | 120 | 14 | 0 | 0 | 1 | 0 | 0 | 14277 | 3090 ms |
| claude-haiku-5-5 | ldf-tab | 135 | 93.3% | 126 | 8 | 0 | 0 | 1 | 0 | 0 | 17264 | 2960 ms |
| claude-haiku-5-5 | toon | 135 | 92.6% | 125 | 6 | 0 | 0 | 4 | 0 | 0 | 26783 | 3348 ms |
| - | csv | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | json | lookup | 15 | 100.0% |
| claude-haiku-5-5 | json | filter | 15 | 100.0% |
| claude-haiku-5-5 | json | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | json | compare | 15 | 100.0% |
| claude-haiku-5-5 | json | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | json | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | json | child_count | 15 | 100.0% |
| claude-haiku-5-5 | json | child_aggregate | 15 | 80.0% |
| claude-haiku-5-5 | json | aggregate_where | 15 | 93.3% |
| claude-haiku-5-5 | json | rank | 8 | 87.5% |
| claude-haiku-5-5 | ldf | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf | compare | 15 | 80.0% |
| claude-haiku-5-5 | ldf | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf | child_count | 15 | 100.0% |
| claude-haiku-5-5 | ldf | child_aggregate | 15 | 73.3% |
| claude-haiku-5-5 | ldf | aggregate_where | 15 | 73.3% |
| claude-haiku-5-5 | ldf | rank | 8 | 100.0% |
| claude-haiku-5-5 | ldf-global | lookup | 15 | 86.7% |
| claude-haiku-5-5 | ldf-global | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf-global | compare | 15 | 53.3% |
| claude-haiku-5-5 | ldf-global | missing_value | 15 | 0.0% |
| claude-haiku-5-5 | ldf-global | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf-global | child_count | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | child_aggregate | 15 | 53.3% |
| claude-haiku-5-5 | ldf-global | aggregate_where | 15 | 86.7% |
| claude-haiku-5-5 | ldf-global | rank | 8 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | compare | 15 | 46.7% |
| claude-haiku-5-5 | ldf-hoisted | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | child_count | 15 | 100.0% |
| claude-haiku-5-5 | ldf-hoisted | child_aggregate | 15 | 66.7% |
| claude-haiku-5-5 | ldf-hoisted | aggregate_where | 15 | 86.7% |
| claude-haiku-5-5 | ldf-hoisted | rank | 8 | 100.0% |
| claude-haiku-5-5 | ldf-tab | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf-tab | compare | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf-tab | child_count | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | child_aggregate | 15 | 66.7% |
| claude-haiku-5-5 | ldf-tab | aggregate_where | 15 | 73.3% |
| claude-haiku-5-5 | ldf-tab | rank | 8 | 100.0% |
| claude-haiku-5-5 | toon | lookup | 15 | 93.3% |
| claude-haiku-5-5 | toon | filter | 15 | 100.0% |
| claude-haiku-5-5 | toon | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | toon | compare | 15 | 86.7% |
| claude-haiku-5-5 | toon | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | toon | nested_extract | 15 | 93.3% |
| claude-haiku-5-5 | toon | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | toon | child_count | 15 | 100.0% |
| claude-haiku-5-5 | toon | child_aggregate | 15 | 66.7% |
| claude-haiku-5-5 | toon | aggregate_where | 15 | 93.3% |
| claude-haiku-5-5 | toon | rank | 8 | 100.0% |
