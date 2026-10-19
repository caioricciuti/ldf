# LLM comprehension results

| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deepseek-chat | json | 82 | 100.0% | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 5326 | 981 ms |
| deepseek-chat | ldf | 82 | 100.0% | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 3994 | 879 ms |
| deepseek-chat | ldf-global | 82 | 78.0% | 64 | 18 | 0 | 0 | 0 | 0 | 0 | 3579 | 908 ms |
| deepseek-chat | ldf-tab | 82 | 100.0% | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 4438 | 979 ms |
| deepseek-chat | ldf-v0 | 82 | 100.0% | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 3625 | 872 ms |
| deepseek-chat | toon | 82 | 100.0% | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 6639 | 993 ms |
| - | csv | 0 | format unsupported: record 0 field "value" is a object (nested) | | | | | | | | |

## By task kind

| model | format | kind | n | accuracy |
|---|---|---|---:|---:|
| deepseek-chat | json | lookup | 15 | 100.0% |
| deepseek-chat | json | filter | 15 | 100.0% |
| deepseek-chat | json | aggregate | 5 | 100.0% |
| deepseek-chat | json | compare | 15 | 100.0% |
| deepseek-chat | json | missing_value | 15 | 100.0% |
| deepseek-chat | json | nested_extract | 15 | 100.0% |
| deepseek-chat | json | multi_record | 2 | 100.0% |
| deepseek-chat | ldf | lookup | 15 | 100.0% |
| deepseek-chat | ldf | filter | 15 | 100.0% |
| deepseek-chat | ldf | aggregate | 5 | 100.0% |
| deepseek-chat | ldf | compare | 15 | 100.0% |
| deepseek-chat | ldf | missing_value | 15 | 100.0% |
| deepseek-chat | ldf | nested_extract | 15 | 100.0% |
| deepseek-chat | ldf | multi_record | 2 | 100.0% |
| deepseek-chat | ldf-global | lookup | 15 | 80.0% |
| deepseek-chat | ldf-global | filter | 15 | 100.0% |
| deepseek-chat | ldf-global | aggregate | 5 | 100.0% |
| deepseek-chat | ldf-global | compare | 15 | 100.0% |
| deepseek-chat | ldf-global | missing_value | 15 | 0.0% |
| deepseek-chat | ldf-global | nested_extract | 15 | 100.0% |
| deepseek-chat | ldf-global | multi_record | 2 | 100.0% |
| deepseek-chat | ldf-tab | lookup | 15 | 100.0% |
| deepseek-chat | ldf-tab | filter | 15 | 100.0% |
| deepseek-chat | ldf-tab | aggregate | 5 | 100.0% |
| deepseek-chat | ldf-tab | compare | 15 | 100.0% |
| deepseek-chat | ldf-tab | missing_value | 15 | 100.0% |
| deepseek-chat | ldf-tab | nested_extract | 15 | 100.0% |
| deepseek-chat | ldf-tab | multi_record | 2 | 100.0% |
| deepseek-chat | ldf-v0 | lookup | 15 | 100.0% |
| deepseek-chat | ldf-v0 | filter | 15 | 100.0% |
| deepseek-chat | ldf-v0 | aggregate | 5 | 100.0% |
| deepseek-chat | ldf-v0 | compare | 15 | 100.0% |
| deepseek-chat | ldf-v0 | missing_value | 15 | 100.0% |
| deepseek-chat | ldf-v0 | nested_extract | 15 | 100.0% |
| deepseek-chat | ldf-v0 | multi_record | 2 | 100.0% |
| deepseek-chat | toon | lookup | 15 | 100.0% |
| deepseek-chat | toon | filter | 15 | 100.0% |
| deepseek-chat | toon | aggregate | 5 | 100.0% |
| deepseek-chat | toon | compare | 15 | 100.0% |
| deepseek-chat | toon | missing_value | 15 | 100.0% |
| deepseek-chat | toon | nested_extract | 15 | 100.0% |
| deepseek-chat | toon | multi_record | 2 | 100.0% |
