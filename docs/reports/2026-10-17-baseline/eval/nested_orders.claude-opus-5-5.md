# LLM comprehension results

| model | format | n | accuracy | correct | wrong | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-opus-5-5 | json | 20 | 95.0% | 19 | 0 | 0 | 1 | 0 | 0 | 7733 | 1972 ms |
| claude-opus-5-5 | json-pretty | 20 | 95.0% | 19 | 1 | 0 | 0 | 0 | 0 | 11615 | 2068 ms |
| claude-opus-5-5 | jsonl | 20 | 95.0% | 19 | 1 | 0 | 0 | 0 | 0 | 7783 | 1719 ms |
| claude-opus-5-5 | ldf | 20 | 100.0% | 20 | 0 | 0 | 0 | 0 | 0 | 4560 | 2109 ms |
| claude-opus-5-5 | toon | 20 | 95.0% | 19 | 1 | 0 | 0 | 0 | 0 | 6835 | 1718 ms |
| claude-opus-5-5 | toon-tab | 20 | 95.0% | 19 | 1 | 0 | 0 | 0 | 0 | 6889 | 1722 ms |
| - | csv | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |
| - | tsv | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |
| - | markdown | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-opus-5-5 | json | lookup | 3 | 100.0% |
| claude-opus-5-5 | json | filter | 3 | 100.0% |
| claude-opus-5-5 | json | aggregate | 3 | 66.7% |
| claude-opus-5-5 | json | compare | 3 | 100.0% |
| claude-opus-5-5 | json | missing_value | 3 | 100.0% |
| claude-opus-5-5 | json | nested_extract | 3 | 100.0% |
| claude-opus-5-5 | json | multi_record | 2 | 100.0% |
| claude-opus-5-5 | json-pretty | lookup | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | filter | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | aggregate | 3 | 66.7% |
| claude-opus-5-5 | json-pretty | compare | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | missing_value | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | nested_extract | 3 | 100.0% |
| claude-opus-5-5 | json-pretty | multi_record | 2 | 100.0% |
| claude-opus-5-5 | jsonl | lookup | 3 | 100.0% |
| claude-opus-5-5 | jsonl | filter | 3 | 100.0% |
| claude-opus-5-5 | jsonl | aggregate | 3 | 66.7% |
| claude-opus-5-5 | jsonl | compare | 3 | 100.0% |
| claude-opus-5-5 | jsonl | missing_value | 3 | 100.0% |
| claude-opus-5-5 | jsonl | nested_extract | 3 | 100.0% |
| claude-opus-5-5 | jsonl | multi_record | 2 | 100.0% |
| claude-opus-5-5 | ldf | lookup | 3 | 100.0% |
| claude-opus-5-5 | ldf | filter | 3 | 100.0% |
| claude-opus-5-5 | ldf | aggregate | 3 | 100.0% |
| claude-opus-5-5 | ldf | compare | 3 | 100.0% |
| claude-opus-5-5 | ldf | missing_value | 3 | 100.0% |
| claude-opus-5-5 | ldf | nested_extract | 3 | 100.0% |
| claude-opus-5-5 | ldf | multi_record | 2 | 100.0% |
| claude-opus-5-5 | toon | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon | filter | 3 | 100.0% |
| claude-opus-5-5 | toon | aggregate | 3 | 66.7% |
| claude-opus-5-5 | toon | compare | 3 | 100.0% |
| claude-opus-5-5 | toon | missing_value | 3 | 100.0% |
| claude-opus-5-5 | toon | nested_extract | 3 | 100.0% |
| claude-opus-5-5 | toon | multi_record | 2 | 100.0% |
| claude-opus-5-5 | toon-tab | lookup | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | filter | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | aggregate | 3 | 66.7% |
| claude-opus-5-5 | toon-tab | compare | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | missing_value | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | nested_extract | 3 | 100.0% |
| claude-opus-5-5 | toon-tab | multi_record | 2 | 100.0% |
