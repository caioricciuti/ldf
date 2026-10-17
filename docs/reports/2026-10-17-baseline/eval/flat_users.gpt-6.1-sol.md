# LLM comprehension results

| model | format | n | accuracy | correct | wrong | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| gpt-6.1-sol | csv | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 1534 | 1271 ms |
| gpt-6.1-sol | json | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 2447 | 1567 ms |
| gpt-6.1-sol | json-pretty | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 3798 | 1371 ms |
| gpt-6.1-sol | jsonl | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 2495 | 1379 ms |
| gpt-6.1-sol | ldf | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 1545 | 1522 ms |
| gpt-6.1-sol | markdown | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 1858 | 1421 ms |
| gpt-6.1-sol | toon | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 1638 | 1210 ms |
| gpt-6.1-sol | toon-tab | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 1633 | 1346 ms |
| gpt-6.1-sol | tsv | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 1528 | 1386 ms |
| gpt-6.1-sol | yaml | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 3026 | 1709 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| gpt-6.1-sol | csv | lookup | 3 | 100.0% |
| gpt-6.1-sol | csv | filter | 3 | 100.0% |
| gpt-6.1-sol | csv | aggregate | 3 | 100.0% |
| gpt-6.1-sol | csv | compare | 3 | 100.0% |
| gpt-6.1-sol | csv | multi_record | 3 | 100.0% |
| gpt-6.1-sol | json | lookup | 3 | 100.0% |
| gpt-6.1-sol | json | filter | 3 | 100.0% |
| gpt-6.1-sol | json | aggregate | 3 | 100.0% |
| gpt-6.1-sol | json | compare | 3 | 100.0% |
| gpt-6.1-sol | json | multi_record | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | lookup | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | filter | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | aggregate | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | compare | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | multi_record | 3 | 100.0% |
| gpt-6.1-sol | jsonl | lookup | 3 | 100.0% |
| gpt-6.1-sol | jsonl | filter | 3 | 100.0% |
| gpt-6.1-sol | jsonl | aggregate | 3 | 100.0% |
| gpt-6.1-sol | jsonl | compare | 3 | 100.0% |
| gpt-6.1-sol | jsonl | multi_record | 3 | 100.0% |
| gpt-6.1-sol | ldf | lookup | 3 | 100.0% |
| gpt-6.1-sol | ldf | filter | 3 | 100.0% |
| gpt-6.1-sol | ldf | aggregate | 3 | 100.0% |
| gpt-6.1-sol | ldf | compare | 3 | 100.0% |
| gpt-6.1-sol | ldf | multi_record | 3 | 100.0% |
| gpt-6.1-sol | markdown | lookup | 3 | 100.0% |
| gpt-6.1-sol | markdown | filter | 3 | 100.0% |
| gpt-6.1-sol | markdown | aggregate | 3 | 100.0% |
| gpt-6.1-sol | markdown | compare | 3 | 100.0% |
| gpt-6.1-sol | markdown | multi_record | 3 | 100.0% |
| gpt-6.1-sol | toon | lookup | 3 | 100.0% |
| gpt-6.1-sol | toon | filter | 3 | 100.0% |
| gpt-6.1-sol | toon | aggregate | 3 | 100.0% |
| gpt-6.1-sol | toon | compare | 3 | 100.0% |
| gpt-6.1-sol | toon | multi_record | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | lookup | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | filter | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | aggregate | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | compare | 3 | 100.0% |
| gpt-6.1-sol | toon-tab | multi_record | 3 | 100.0% |
| gpt-6.1-sol | tsv | lookup | 3 | 100.0% |
| gpt-6.1-sol | tsv | filter | 3 | 100.0% |
| gpt-6.1-sol | tsv | aggregate | 3 | 100.0% |
| gpt-6.1-sol | tsv | compare | 3 | 100.0% |
| gpt-6.1-sol | tsv | multi_record | 3 | 100.0% |
| gpt-6.1-sol | yaml | lookup | 3 | 100.0% |
| gpt-6.1-sol | yaml | filter | 3 | 100.0% |
| gpt-6.1-sol | yaml | aggregate | 3 | 100.0% |
| gpt-6.1-sol | yaml | compare | 3 | 100.0% |
| gpt-6.1-sol | yaml | multi_record | 3 | 100.0% |
