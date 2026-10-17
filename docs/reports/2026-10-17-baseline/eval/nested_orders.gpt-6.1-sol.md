# LLM comprehension results

| model | format | n | accuracy | correct | wrong | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| gpt-6.1-sol | json | 20 | 100.0% | 20 | 0 | 0 | 0 | 0 | 0 | 5176 | 1815 ms |
| gpt-6.1-sol | json-pretty | 20 | 100.0% | 20 | 0 | 0 | 0 | 0 | 0 | 9091 | 1743 ms |
| gpt-6.1-sol | jsonl | 20 | 100.0% | 20 | 0 | 0 | 0 | 0 | 0 | 5224 | 1719 ms |
| gpt-6.1-sol | toon | 20 | 100.0% | 20 | 0 | 0 | 0 | 0 | 0 | 5549 | 1755 ms |
| gpt-6.1-sol | toon-tab | 20 | 100.0% | 20 | 0 | 0 | 0 | 0 | 0 | 5551 | 1714 ms |
| - | csv | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |
| - | tsv | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |
| - | markdown | 0 | format unsupported: record 0 field "customer" is a object (nested) | | | | | | | | |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| gpt-6.1-sol | json | lookup | 3 | 100.0% |
| gpt-6.1-sol | json | filter | 3 | 100.0% |
| gpt-6.1-sol | json | aggregate | 3 | 100.0% |
| gpt-6.1-sol | json | compare | 3 | 100.0% |
| gpt-6.1-sol | json | missing_value | 3 | 100.0% |
| gpt-6.1-sol | json | nested_extract | 3 | 100.0% |
| gpt-6.1-sol | json | multi_record | 2 | 100.0% |
| gpt-6.1-sol | json-pretty | lookup | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | filter | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | aggregate | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | compare | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | missing_value | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | nested_extract | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | multi_record | 2 | 100.0% |
| gpt-6.1-sol | jsonl | lookup | 3 | 100.0% |
| gpt-6.1-sol | jsonl | filter | 3 | 100.0% |
| gpt-6.1-sol | jsonl | aggregate | 3 | 100.0% |
| gpt-6.1-sol | jsonl | compare | 3 | 100.0% |
| gpt-6.1-sol | jsonl | missing_value | 3 | 100.0% |
| gpt-6.1-sol | jsonl | nested_extract | 3 | 100.0% |
| gpt-6.1-sol | jsonl | multi_record | 2 | 100.0% |
| gpt-6.1-sol | toon | lookup | 3 | 100.0% |
| gpt-6.1-sol | toon | filter | 3 | 100.0% |
| gpt-6.1-sol | toon | aggregate | 3 | 100.0% |
| gpt-6.1-sol | toon | compare | 3 | 100.0% |
| gpt-6.1-sol | toon | missing_value | 3 | 100.0% |
| gpt-6.1-sol | toon | nested_extract | 3 | 100.0% |
| gpt-6.1-sol | toon | multi_record | 2 | 100.0% |
| gpt-6.1-sol | toon-tab | lookup | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | filter | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | aggregate | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | compare | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | missing_value | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | nested_extract | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | multi_record | 2 | 100.0% |
