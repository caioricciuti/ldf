# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| gpt-6.1-sol | csv | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 1798 | 1385 ms |
| gpt-6.1-sol | json | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 2827 | 1483 ms |
| gpt-6.1-sol | json-pretty | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 4166 | 1382 ms |
| gpt-6.1-sol | jsonl | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 2875 | 1359 ms |
| gpt-6.1-sol | ldf | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 1885 | 1524 ms |
| gpt-6.1-sol | markdown | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 2832 | 1341 ms |
| gpt-6.1-sol | toon | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 3437 | 1439 ms |
| gpt-6.1-sol | toon-tab | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 3439 | 1495 ms |
| gpt-6.1-sol | tsv | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 1790 | 1403 ms |
| gpt-6.1-sol | yaml | 18 | 100.0% | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 3372 | 1728 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| gpt-6.1-sol | csv | lookup | 3 | 100.0% |
| gpt-6.1-sol | csv | filter | 3 | 100.0% |
| gpt-6.1-sol | csv | aggregate | 3 | 100.0% |
| gpt-6.1-sol | csv | compare | 3 | 100.0% |
| gpt-6.1-sol | csv | missing_value | 3 | 100.0% |
| gpt-6.1-sol | csv | multi_record | 3 | 100.0% |
| gpt-6.1-sol | json | lookup | 3 | 100.0% |
| gpt-6.1-sol | json | filter | 3 | 100.0% |
| gpt-6.1-sol | json | aggregate | 3 | 100.0% |
| gpt-6.1-sol | json | compare | 3 | 100.0% |
| gpt-6.1-sol | json | missing_value | 3 | 100.0% |
| gpt-6.1-sol | json | multi_record | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | lookup | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | filter | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | aggregate | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | compare | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | missing_value | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | multi_record | 3 | 100.0% |
| gpt-6.1-sol | jsonl | lookup | 3 | 100.0% |
| gpt-6.1-sol | jsonl | filter | 3 | 100.0% |
| gpt-6.1-sol | jsonl | aggregate | 3 | 100.0% |
| gpt-6.1-sol | jsonl | compare | 3 | 100.0% |
| gpt-6.1-sol | jsonl | missing_value | 3 | 100.0% |
| gpt-6.1-sol | jsonl | multi_record | 3 | 100.0% |
| gpt-6.1-sol | ldf | lookup | 3 | 100.0% |
| gpt-6.1-sol | ldf | filter | 3 | 100.0% |
| gpt-6.1-sol | ldf | aggregate | 3 | 100.0% |
| gpt-6.1-sol | ldf | compare | 3 | 100.0% |
| gpt-6.1-sol | ldf | missing_value | 3 | 100.0% |
| gpt-6.1-sol | ldf | multi_record | 3 | 100.0% |
| gpt-6.1-sol | markdown | lookup | 3 | 100.0% |
| gpt-6.1-sol | markdown | filter | 3 | 100.0% |
| gpt-6.1-sol | markdown | aggregate | 3 | 100.0% |
| gpt-6.1-sol | markdown | compare | 3 | 100.0% |
| gpt-6.1-sol | markdown | missing_value | 3 | 100.0% |
| gpt-6.1-sol | markdown | multi_record | 3 | 100.0% |
| gpt-6.1-sol | toon | lookup | 3 | 100.0% |
| gpt-6.1-sol | toon | filter | 3 | 100.0% |
| gpt-6.1-sol | toon | aggregate | 3 | 100.0% |
| gpt-6.1-sol | toon | compare | 3 | 100.0% |
| gpt-6.1-sol | toon | missing_value | 3 | 100.0% |
| gpt-6.1-sol | toon | multi_record | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | lookup | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | filter | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | aggregate | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | compare | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | missing_value | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | multi_record | 3 | 100.0% |
| gpt-6.1-sol | tsv | lookup | 3 | 100.0% |
| gpt-6.1-sol | tsv | filter | 3 | 100.0% |
| gpt-6.1-sol | tsv | aggregate | 3 | 100.0% |
| gpt-6.1-sol | tsv | compare | 3 | 100.0% |
| gpt-6.1-sol | tsv | missing_value | 3 | 100.0% |
| gpt-6.1-sol | tsv | multi_record | 3 | 100.0% |
| gpt-6.1-sol | yaml | lookup | 3 | 100.0% |
| gpt-6.1-sol | yaml | filter | 3 | 100.0% |
| gpt-6.1-sol | yaml | aggregate | 3 | 100.0% |
| gpt-6.1-sol | yaml | compare | 3 | 100.0% |
| gpt-6.1-sol | yaml | missing_value | 3 | 100.0% |
| gpt-6.1-sol | yaml | multi_record | 3 | 100.0% |
