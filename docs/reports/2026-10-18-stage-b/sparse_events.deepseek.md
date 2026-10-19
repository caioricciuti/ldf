# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | csv | 115 | 47.8% | 55 | 60 | 0 | 0 | 0 | 0 | 0 | 7197 | 951 ms |
| deepseek-chat | json | 115 | 60.9% | 70 | 45 | 0 | 0 | 0 | 0 | 0 | 11388 | 957 ms |
| deepseek-chat | ldf | 115 | 52.2% | 60 | 55 | 0 | 0 | 0 | 0 | 0 | 8875 | 974 ms |
| deepseek-chat | ldf-global | 115 | 40.9% | 47 | 68 | 0 | 0 | 0 | 0 | 0 | 7260 | 1022 ms |
| deepseek-chat | ldf-tab | 115 | 56.5% | 65 | 50 | 0 | 0 | 0 | 0 | 0 | 9008 | 983 ms |
| deepseek-chat | ldf-v0 | 115 | 48.7% | 56 | 59 | 0 | 0 | 0 | 0 | 0 | 7480 | 948 ms |
| deepseek-chat | toon | 115 | 62.6% | 72 | 43 | 0 | 0 | 0 | 0 | 0 | 13745 | 1014 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | csv | lookup | 15 | 100.0% |
| deepseek-chat | csv | filter | 15 | 6.7% |
| deepseek-chat | csv | aggregate | 15 | 26.7% |
| deepseek-chat | csv | compare | 15 | 86.7% |
| deepseek-chat | csv | missing_value | 15 | 86.7% |
| deepseek-chat | csv | multi_record | 10 | 20.0% |
| deepseek-chat | csv | aggregate_where | 15 | 33.3% |
| deepseek-chat | csv | rank | 15 | 13.3% |
| deepseek-chat | json | lookup | 15 | 100.0% |
| deepseek-chat | json | filter | 15 | 6.7% |
| deepseek-chat | json | aggregate | 15 | 33.3% |
| deepseek-chat | json | compare | 15 | 86.7% |
| deepseek-chat | json | missing_value | 15 | 86.7% |
| deepseek-chat | json | multi_record | 10 | 60.0% |
| deepseek-chat | json | aggregate_where | 15 | 46.7% |
| deepseek-chat | json | rank | 15 | 66.7% |
| deepseek-chat | ldf | lookup | 15 | 100.0% |
| deepseek-chat | ldf | filter | 15 | 13.3% |
| deepseek-chat | ldf | aggregate | 15 | 33.3% |
| deepseek-chat | ldf | compare | 15 | 86.7% |
| deepseek-chat | ldf | missing_value | 15 | 80.0% |
| deepseek-chat | ldf | multi_record | 10 | 30.0% |
| deepseek-chat | ldf | aggregate_where | 15 | 26.7% |
| deepseek-chat | ldf | rank | 15 | 40.0% |
| deepseek-chat | ldf-global | lookup | 15 | 73.3% |
| deepseek-chat | ldf-global | filter | 15 | 6.7% |
| deepseek-chat | ldf-global | aggregate | 15 | 20.0% |
| deepseek-chat | ldf-global | compare | 15 | 80.0% |
| deepseek-chat | ldf-global | missing_value | 15 | 80.0% |
| deepseek-chat | ldf-global | multi_record | 10 | 20.0% |
| deepseek-chat | ldf-global | aggregate_where | 15 | 20.0% |
| deepseek-chat | ldf-global | rank | 15 | 20.0% |
| deepseek-chat | ldf-tab | lookup | 15 | 100.0% |
| deepseek-chat | ldf-tab | filter | 15 | 26.7% |
| deepseek-chat | ldf-tab | aggregate | 15 | 40.0% |
| deepseek-chat | ldf-tab | compare | 15 | 86.7% |
| deepseek-chat | ldf-tab | missing_value | 15 | 80.0% |
| deepseek-chat | ldf-tab | multi_record | 10 | 30.0% |
| deepseek-chat | ldf-tab | aggregate_where | 15 | 26.7% |
| deepseek-chat | ldf-tab | rank | 15 | 53.3% |
| deepseek-chat | ldf-v0 | lookup | 15 | 100.0% |
| deepseek-chat | ldf-v0 | filter | 15 | 13.3% |
| deepseek-chat | ldf-v0 | aggregate | 15 | 26.7% |
| deepseek-chat | ldf-v0 | compare | 15 | 93.3% |
| deepseek-chat | ldf-v0 | missing_value | 15 | 93.3% |
| deepseek-chat | ldf-v0 | multi_record | 10 | 20.0% |
| deepseek-chat | ldf-v0 | aggregate_where | 15 | 20.0% |
| deepseek-chat | ldf-v0 | rank | 15 | 13.3% |
| deepseek-chat | toon | lookup | 15 | 100.0% |
| deepseek-chat | toon | filter | 15 | 6.7% |
| deepseek-chat | toon | aggregate | 15 | 33.3% |
| deepseek-chat | toon | compare | 15 | 93.3% |
| deepseek-chat | toon | missing_value | 15 | 93.3% |
| deepseek-chat | toon | multi_record | 10 | 80.0% |
| deepseek-chat | toon | aggregate_where | 15 | 26.7% |
| deepseek-chat | toon | rank | 15 | 73.3% |
