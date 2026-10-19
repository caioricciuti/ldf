# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | json | 135 | 62.2% | 84 | 51 | 0 | 0 | 0 | 0 | 0 | 21050 | 1128 ms |
| deepseek-chat | ldf | 135 | 62.2% | 84 | 51 | 0 | 0 | 0 | 0 | 0 | 13294 | 1008 ms |
| deepseek-chat | ldf-global | 135 | 52.6% | 71 | 64 | 0 | 0 | 0 | 0 | 0 | 11567 | 978 ms |
| deepseek-chat | ldf-hoisted | 135 | 60.0% | 81 | 54 | 0 | 0 | 0 | 0 | 0 | 11398 | 998 ms |
| deepseek-chat | ldf-tab | 135 | 67.4% | 91 | 44 | 0 | 0 | 0 | 0 | 0 | 13732 | 1006 ms |
| deepseek-chat | toon | 135 | 64.4% | 87 | 48 | 0 | 0 | 0 | 0 | 0 | 21976 | 1173 ms |
| - | csv | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | json | lookup | 15 | 100.0% |
| deepseek-chat | json | filter | 15 | 6.7% |
| deepseek-chat | json | aggregate | 5 | 60.0% |
| deepseek-chat | json | compare | 15 | 86.7% |
| deepseek-chat | json | missing_value | 15 | 100.0% |
| deepseek-chat | json | nested_extract | 15 | 100.0% |
| deepseek-chat | json | multi_record | 2 | 100.0% |
| deepseek-chat | json | child_count | 15 | 100.0% |
| deepseek-chat | json | child_aggregate | 15 | 0.0% |
| deepseek-chat | json | aggregate_where | 15 | 13.3% |
| deepseek-chat | json | rank | 8 | 37.5% |
| deepseek-chat | ldf | lookup | 15 | 100.0% |
| deepseek-chat | ldf | filter | 15 | 6.7% |
| deepseek-chat | ldf | aggregate | 5 | 60.0% |
| deepseek-chat | ldf | compare | 15 | 86.7% |
| deepseek-chat | ldf | missing_value | 15 | 100.0% |
| deepseek-chat | ldf | nested_extract | 15 | 100.0% |
| deepseek-chat | ldf | multi_record | 2 | 100.0% |
| deepseek-chat | ldf | child_count | 15 | 93.3% |
| deepseek-chat | ldf | child_aggregate | 15 | 0.0% |
| deepseek-chat | ldf | aggregate_where | 15 | 6.7% |
| deepseek-chat | ldf | rank | 8 | 62.5% |
| deepseek-chat | ldf-global | lookup | 15 | 86.7% |
| deepseek-chat | ldf-global | filter | 15 | 20.0% |
| deepseek-chat | ldf-global | aggregate | 5 | 60.0% |
| deepseek-chat | ldf-global | compare | 15 | 80.0% |
| deepseek-chat | ldf-global | missing_value | 15 | 0.0% |
| deepseek-chat | ldf-global | nested_extract | 15 | 100.0% |
| deepseek-chat | ldf-global | multi_record | 2 | 100.0% |
| deepseek-chat | ldf-global | child_count | 15 | 86.7% |
| deepseek-chat | ldf-global | child_aggregate | 15 | 6.7% |
| deepseek-chat | ldf-global | aggregate_where | 15 | 13.3% |
| deepseek-chat | ldf-global | rank | 8 | 87.5% |
| deepseek-chat | ldf-hoisted | lookup | 15 | 100.0% |
| deepseek-chat | ldf-hoisted | filter | 15 | 6.7% |
| deepseek-chat | ldf-hoisted | aggregate | 5 | 60.0% |
| deepseek-chat | ldf-hoisted | compare | 15 | 86.7% |
| deepseek-chat | ldf-hoisted | missing_value | 15 | 100.0% |
| deepseek-chat | ldf-hoisted | nested_extract | 15 | 100.0% |
| deepseek-chat | ldf-hoisted | multi_record | 2 | 100.0% |
| deepseek-chat | ldf-hoisted | child_count | 15 | 80.0% |
| deepseek-chat | ldf-hoisted | child_aggregate | 15 | 0.0% |
| deepseek-chat | ldf-hoisted | aggregate_where | 15 | 13.3% |
| deepseek-chat | ldf-hoisted | rank | 8 | 37.5% |
| deepseek-chat | ldf-tab | lookup | 15 | 100.0% |
| deepseek-chat | ldf-tab | filter | 15 | 20.0% |
| deepseek-chat | ldf-tab | aggregate | 5 | 60.0% |
| deepseek-chat | ldf-tab | compare | 15 | 100.0% |
| deepseek-chat | ldf-tab | missing_value | 15 | 100.0% |
| deepseek-chat | ldf-tab | nested_extract | 15 | 86.7% |
| deepseek-chat | ldf-tab | multi_record | 2 | 50.0% |
| deepseek-chat | ldf-tab | child_count | 15 | 100.0% |
| deepseek-chat | ldf-tab | child_aggregate | 15 | 6.7% |
| deepseek-chat | ldf-tab | aggregate_where | 15 | 20.0% |
| deepseek-chat | ldf-tab | rank | 8 | 87.5% |
| deepseek-chat | toon | lookup | 15 | 100.0% |
| deepseek-chat | toon | filter | 15 | 6.7% |
| deepseek-chat | toon | aggregate | 5 | 60.0% |
| deepseek-chat | toon | compare | 15 | 93.3% |
| deepseek-chat | toon | missing_value | 15 | 100.0% |
| deepseek-chat | toon | nested_extract | 15 | 93.3% |
| deepseek-chat | toon | multi_record | 2 | 100.0% |
| deepseek-chat | toon | child_count | 15 | 100.0% |
| deepseek-chat | toon | child_aggregate | 15 | 6.7% |
| deepseek-chat | toon | aggregate_where | 15 | 13.3% |
| deepseek-chat | toon | rank | 8 | 62.5% |
