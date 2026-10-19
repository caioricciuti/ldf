# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | json | 82 | 98.8% | 81 | 1 | 0 | 0 | 0 | 0 | 0 | 7843 | 914 ms |
| claude-haiku-5-5 | ldf | 82 | 96.3% | 79 | 3 | 0 | 0 | 0 | 0 | 0 | 5972 | 949 ms |
| claude-haiku-5-5 | ldf-global | 82 | 91.5% | 75 | 7 | 0 | 0 | 0 | 0 | 0 | 5142 | 1117 ms |
| claude-haiku-5-5 | ldf-tab | 82 | 98.8% | 81 | 1 | 0 | 0 | 0 | 0 | 0 | 6261 | 992 ms |
| claude-haiku-5-5 | ldf-v0 | 82 | 96.3% | 79 | 3 | 0 | 0 | 0 | 0 | 0 | 5214 | 926 ms |
| claude-haiku-5-5 | toon | 82 | 100.0% | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 7947 | 912 ms |
| - | csv | 0 | format unsupported: record 0 field "value" is a object (nested) | | | | | | | | |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | json | lookup | 15 | 100.0% |
| claude-haiku-5-5 | json | filter | 15 | 100.0% |
| claude-haiku-5-5 | json | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | json | compare | 15 | 93.3% |
| claude-haiku-5-5 | json | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | json | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf | lookup | 15 | 93.3% |
| claude-haiku-5-5 | ldf | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf | compare | 15 | 86.7% |
| claude-haiku-5-5 | ldf | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf-global | lookup | 15 | 73.3% |
| claude-haiku-5-5 | ldf-global | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf-global | compare | 15 | 93.3% |
| claude-haiku-5-5 | ldf-global | missing_value | 15 | 86.7% |
| claude-haiku-5-5 | ldf-global | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf-global | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf-tab | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf-tab | compare | 15 | 93.3% |
| claude-haiku-5-5 | ldf-tab | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | filter | 15 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | compare | 15 | 80.0% |
| claude-haiku-5-5 | ldf-v0 | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | multi_record | 2 | 100.0% |
| claude-haiku-5-5 | toon | lookup | 15 | 100.0% |
| claude-haiku-5-5 | toon | filter | 15 | 100.0% |
| claude-haiku-5-5 | toon | aggregate | 5 | 100.0% |
| claude-haiku-5-5 | toon | compare | 15 | 100.0% |
| claude-haiku-5-5 | toon | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | toon | nested_extract | 15 | 100.0% |
| claude-haiku-5-5 | toon | multi_record | 2 | 100.0% |
