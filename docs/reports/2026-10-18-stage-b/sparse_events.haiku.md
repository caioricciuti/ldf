# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-5-5 | csv | 115 | 94.8% | 109 | 5 | 1 | 0 | 0 | 0 | 0 | 9382 | 2900 ms |
| claude-haiku-5-5 | json | 115 | 92.2% | 106 | 9 | 0 | 0 | 0 | 0 | 0 | 15316 | 2322 ms |
| claude-haiku-5-5 | ldf | 115 | 93.9% | 108 | 7 | 0 | 0 | 0 | 0 | 0 | 11442 | 2420 ms |
| claude-haiku-5-5 | ldf-global | 115 | 70.4% | 81 | 11 | 2 | 0 | 21 | 0 | 0 | 9144 | 7023 ms |
| claude-haiku-5-5 | ldf-tab | 115 | 89.6% | 103 | 12 | 0 | 0 | 0 | 0 | 0 | 11235 | 2655 ms |
| claude-haiku-5-5 | ldf-v0 | 115 | 89.6% | 103 | 3 | 1 | 0 | 8 | 0 | 0 | 9660 | 4781 ms |
| claude-haiku-5-5 | toon | 115 | 93.9% | 108 | 7 | 0 | 0 | 0 | 0 | 0 | 16244 | 2304 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| claude-haiku-5-5 | csv | lookup | 15 | 100.0% |
| claude-haiku-5-5 | csv | filter | 15 | 93.3% |
| claude-haiku-5-5 | csv | aggregate | 15 | 80.0% |
| claude-haiku-5-5 | csv | compare | 15 | 93.3% |
| claude-haiku-5-5 | csv | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | csv | multi_record | 10 | 100.0% |
| claude-haiku-5-5 | csv | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | csv | rank | 15 | 93.3% |
| claude-haiku-5-5 | json | lookup | 15 | 100.0% |
| claude-haiku-5-5 | json | filter | 15 | 80.0% |
| claude-haiku-5-5 | json | aggregate | 15 | 93.3% |
| claude-haiku-5-5 | json | compare | 15 | 100.0% |
| claude-haiku-5-5 | json | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | json | multi_record | 10 | 90.0% |
| claude-haiku-5-5 | json | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | json | rank | 15 | 73.3% |
| claude-haiku-5-5 | ldf | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf | filter | 15 | 93.3% |
| claude-haiku-5-5 | ldf | aggregate | 15 | 80.0% |
| claude-haiku-5-5 | ldf | compare | 15 | 100.0% |
| claude-haiku-5-5 | ldf | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf | multi_record | 10 | 100.0% |
| claude-haiku-5-5 | ldf | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | ldf | rank | 15 | 80.0% |
| claude-haiku-5-5 | ldf-global | lookup | 15 | 80.0% |
| claude-haiku-5-5 | ldf-global | filter | 15 | 40.0% |
| claude-haiku-5-5 | ldf-global | aggregate | 15 | 53.3% |
| claude-haiku-5-5 | ldf-global | compare | 15 | 86.7% |
| claude-haiku-5-5 | ldf-global | missing_value | 15 | 80.0% |
| claude-haiku-5-5 | ldf-global | multi_record | 10 | 80.0% |
| claude-haiku-5-5 | ldf-global | aggregate_where | 15 | 93.3% |
| claude-haiku-5-5 | ldf-global | rank | 15 | 53.3% |
| claude-haiku-5-5 | ldf-tab | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | filter | 15 | 66.7% |
| claude-haiku-5-5 | ldf-tab | aggregate | 15 | 80.0% |
| claude-haiku-5-5 | ldf-tab | compare | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf-tab | multi_record | 10 | 90.0% |
| claude-haiku-5-5 | ldf-tab | aggregate_where | 15 | 93.3% |
| claude-haiku-5-5 | ldf-tab | rank | 15 | 86.7% |
| claude-haiku-5-5 | ldf-v0 | lookup | 15 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | filter | 15 | 80.0% |
| claude-haiku-5-5 | ldf-v0 | aggregate | 15 | 66.7% |
| claude-haiku-5-5 | ldf-v0 | compare | 15 | 93.3% |
| claude-haiku-5-5 | ldf-v0 | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | multi_record | 10 | 90.0% |
| claude-haiku-5-5 | ldf-v0 | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | ldf-v0 | rank | 15 | 86.7% |
| claude-haiku-5-5 | toon | lookup | 15 | 93.3% |
| claude-haiku-5-5 | toon | filter | 15 | 86.7% |
| claude-haiku-5-5 | toon | aggregate | 15 | 93.3% |
| claude-haiku-5-5 | toon | compare | 15 | 100.0% |
| claude-haiku-5-5 | toon | missing_value | 15 | 100.0% |
| claude-haiku-5-5 | toon | multi_record | 10 | 90.0% |
| claude-haiku-5-5 | toon | aggregate_where | 15 | 100.0% |
| claude-haiku-5-5 | toon | rank | 15 | 86.7% |
