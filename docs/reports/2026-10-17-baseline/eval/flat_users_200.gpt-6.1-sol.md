# LLM comprehension results

| model | format | n | accuracy | correct | wrong | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| gpt-6.1-sol | json | 15 | 100.0% | 15 | 0 | 0 | 0 | 0 | 0 | 9477 | 4774 ms |
| gpt-6.1-sol | json-pretty | 10 | 90.0% | 9 | 1 | 0 | 0 | 0 | 0 | 14876 | 5218 ms |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| gpt-6.1-sol | json | lookup | 3 | 100.0% |
| gpt-6.1-sol | json | filter | 3 | 100.0% |
| gpt-6.1-sol | json | aggregate | 3 | 100.0% |
| gpt-6.1-sol | json | compare | 3 | 100.0% |
| gpt-6.1-sol | json | multi_record | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | lookup | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | filter | 3 | 100.0% |
| gpt-6.1-sol | json-pretty | aggregate | 3 | 66.7% |
| gpt-6.1-sol | json-pretty | compare | 1 | 100.0% |
