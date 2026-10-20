# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | bare | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | csv | 89 | 61.8% | 55 | 34 | 0 | 0 | 0 | 0 | 0 | 0 | 6159 | 897 ms |
| deepseek-chat | json | 89 | 66.3% | 59 | 30 | 0 | 0 | 0 | 0 | 0 | 0 | 10086 | 1025 ms |
| deepseek-chat | ldf | 89 | 68.5% | 61 | 28 | 0 | 0 | 0 | 0 | 0 | 0 | 6174 | 916 ms |
| deepseek-chat | ldf-global | 89 | 68.5% | 61 | 28 | 0 | 0 | 0 | 0 | 0 | 0 | 6166 | 877 ms |
| deepseek-chat | ldf-tab | 89 | 64.0% | 57 | 32 | 0 | 0 | 0 | 0 | 0 | 0 | 6237 | 955 ms |
| deepseek-chat | toon | 89 | 68.5% | 61 | 28 | 0 | 0 | 0 | 0 | 0 | 0 | 6364 | 904 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | csv | lookup | 15 | 100.0% |
| deepseek-chat | csv | filter | 15 | 40.0% |
| deepseek-chat | csv | aggregate | 15 | 73.3% |
| deepseek-chat | csv | compare | 15 | 66.7% |
| deepseek-chat | csv | multi_record | 5 | 60.0% |
| deepseek-chat | csv | aggregate_where | 15 | 33.3% |
| deepseek-chat | csv | rank | 9 | 55.6% |
| deepseek-chat | json | lookup | 15 | 100.0% |
| deepseek-chat | json | filter | 15 | 46.7% |
| deepseek-chat | json | aggregate | 15 | 73.3% |
| deepseek-chat | json | compare | 15 | 80.0% |
| deepseek-chat | json | multi_record | 5 | 60.0% |
| deepseek-chat | json | aggregate_where | 15 | 33.3% |
| deepseek-chat | json | rank | 9 | 66.7% |
| deepseek-chat | ldf | lookup | 15 | 100.0% |
| deepseek-chat | ldf | filter | 15 | 46.7% |
| deepseek-chat | ldf | aggregate | 15 | 80.0% |
| deepseek-chat | ldf | compare | 15 | 66.7% |
| deepseek-chat | ldf | multi_record | 5 | 80.0% |
| deepseek-chat | ldf | aggregate_where | 15 | 33.3% |
| deepseek-chat | ldf | rank | 9 | 88.9% |
| deepseek-chat | ldf-global | lookup | 15 | 100.0% |
| deepseek-chat | ldf-global | filter | 15 | 46.7% |
| deepseek-chat | ldf-global | aggregate | 15 | 80.0% |
| deepseek-chat | ldf-global | compare | 15 | 66.7% |
| deepseek-chat | ldf-global | multi_record | 5 | 80.0% |
| deepseek-chat | ldf-global | aggregate_where | 15 | 33.3% |
| deepseek-chat | ldf-global | rank | 9 | 88.9% |
| deepseek-chat | ldf-tab | lookup | 15 | 100.0% |
| deepseek-chat | ldf-tab | filter | 15 | 53.3% |
| deepseek-chat | ldf-tab | aggregate | 15 | 80.0% |
| deepseek-chat | ldf-tab | compare | 15 | 66.7% |
| deepseek-chat | ldf-tab | multi_record | 5 | 60.0% |
| deepseek-chat | ldf-tab | aggregate_where | 15 | 26.7% |
| deepseek-chat | ldf-tab | rank | 9 | 55.6% |
| deepseek-chat | toon | lookup | 15 | 100.0% |
| deepseek-chat | toon | filter | 15 | 46.7% |
| deepseek-chat | toon | aggregate | 15 | 80.0% |
| deepseek-chat | toon | compare | 15 | 73.3% |
| deepseek-chat | toon | multi_record | 5 | 80.0% |
| deepseek-chat | toon | aggregate_where | 15 | 33.3% |
| deepseek-chat | toon | rank | 9 | 77.8% |
