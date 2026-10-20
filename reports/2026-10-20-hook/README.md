# Claude Code Bash hook, ldf-mcp 0.4.0

Written by `eval/bench_hook.py`. Tokens are exact o200k_base counts (`ldf-bench count`) of the text the model reads: stdout, plus the hook's note when it converted. "first" is the first conversion in a session (the LDF primer rides along), "again" every later one (a two-line note). Claude counts are not here: its tokenizer is not public. Times are per invocation, process start included, 30 warm calls; "inside" is the hook's own time after start.

Machine: Apple M2 Pro, Darwin 27.0.0.

| dataset | rows | style | JSON bytes | model | first | again | JSON tok | first tok | again tok | again / JSON | wall p50 ms | inside p50 ms |
|---|---:|---|---:|---|---|---|---:|---:|---:|---:|---:|---:|
| flat_users | 20 | compact | 2747 | none | json (not_shorter) | json (not_shorter) | 944 | 944 | 944 | 1.00 | 5.2 | 0.11 |
| flat_users | 20 | compact | 2747 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 944 | 944 | 944 | 1.00 | 5.3 | 0.32 |
| flat_users | 20 | compact | 2747 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 944 | 944 | 944 | 1.00 | 5.4 | 0.32 |
| flat_users | 20 | compact | 2747 | claude-sonnet-5-5 | json (selector) | json (selector) | 944 | 944 | 944 | 1.00 | 5.1 | 0.29 |
| flat_users | 20 | pretty | 3828 | none | ldf | ldf | 1483 | 1075 | 637 | 0.43 | 4.8 | 0.12 |
| flat_users | 20 | pretty | 3828 | claude-opus-5-5 | ldf | ldf | 1483 | 1075 | 637 | 0.43 | 5.8 | 0.35 |
| flat_users | 20 | pretty | 3828 | claude-haiku-5-5 | ldf | ldf | 1483 | 1075 | 637 | 0.43 | 5.1 | 0.32 |
| flat_users | 20 | pretty | 3828 | claude-sonnet-5-5 | json (selector) | json (selector) | 1483 | 1483 | 1483 | 1.00 | 5.6 | 0.30 |
| flat_users | 100 | compact | 13805 | none | ldf | ldf | 4685 | 3345 | 2907 | 0.62 | 4.7 | 0.26 |
| flat_users | 100 | compact | 13805 | claude-opus-5-5 | ldf | ldf | 4685 | 3345 | 2907 | 0.62 | 6.4 | 0.96 |
| flat_users | 100 | compact | 13805 | claude-haiku-5-5 | ldf | ldf | 4685 | 3345 | 2907 | 0.62 | 5.9 | 0.91 |
| flat_users | 100 | compact | 13805 | claude-sonnet-5-5 | json (selector) | json (selector) | 4685 | 4685 | 4685 | 1.00 | 5.7 | 0.88 |
| flat_users | 100 | pretty | 19206 | none | ldf | ldf | 7384 | 3345 | 2907 | 0.39 | 4.8 | 0.32 |
| flat_users | 100 | pretty | 19206 | claude-opus-5-5 | ldf | ldf | 7384 | 3345 | 2907 | 0.39 | 5.9 | 0.99 |
| flat_users | 100 | pretty | 19206 | claude-haiku-5-5 | ldf | ldf | 7384 | 3345 | 2907 | 0.39 | 6.1 | 0.98 |
| flat_users | 100 | pretty | 19206 | claude-sonnet-5-5 | json (selector) | json (selector) | 7384 | 7384 | 7384 | 1.00 | 6.5 | 0.98 |
| nested_orders | 20 | compact | 5998 | none | ldf | ldf | 2021 | 1842 | 1404 | 0.69 | 4.7 | 0.18 |
| nested_orders | 20 | compact | 5998 | claude-opus-5-5 | json (selector) | json (selector) | 2021 | 2021 | 2021 | 1.00 | 5.4 | 0.62 |
| nested_orders | 20 | compact | 5998 | claude-haiku-5-5 | json (selector) | json (selector) | 2021 | 2021 | 2021 | 1.00 | 5.2 | 0.62 |
| nested_orders | 20 | compact | 5998 | claude-sonnet-5-5 | json (selector) | json (selector) | 2021 | 2021 | 2021 | 1.00 | 5.4 | 0.61 |
| nested_orders | 20 | pretty | 10623 | none | ldf | ldf | 3579 | 1842 | 1404 | 0.39 | 5.6 | 0.21 |
| nested_orders | 20 | pretty | 10623 | claude-opus-5-5 | json (selector) | json (selector) | 3579 | 3579 | 3579 | 1.00 | 5.9 | 0.65 |
| nested_orders | 20 | pretty | 10623 | claude-haiku-5-5 | json (selector) | json (selector) | 3579 | 3579 | 3579 | 1.00 | 5.8 | 0.66 |
| nested_orders | 20 | pretty | 10623 | claude-sonnet-5-5 | json (selector) | json (selector) | 3579 | 3579 | 3579 | 1.00 | 6.8 | 0.65 |
| nested_orders | 100 | compact | 29475 | none | ldf | ldf | 9914 | 6889 | 6451 | 0.65 | 6.5 | 0.74 |
| nested_orders | 100 | compact | 29475 | claude-opus-5-5 | ldf | ldf | 9914 | 5950 | 5512 | 0.56 | 8.1 | 2.49 |
| nested_orders | 100 | compact | 29475 | claude-haiku-5-5 | json (selector) | json (selector) | 9914 | 9914 | 9914 | 1.00 | 8.6 | 2.56 |
| nested_orders | 100 | compact | 29475 | claude-sonnet-5-5 | json (selector) | json (selector) | 9914 | 9914 | 9914 | 1.00 | 8.9 | 2.60 |
| nested_orders | 100 | pretty | 52068 | none | json (persisted) | json (persisted) | 17543 | 17543 | 17543 | 1.00 | 6.4 | 0.53 |
| nested_orders | 100 | pretty | 52068 | claude-opus-5-5 | json (persisted) | json (persisted) | 17543 | 17543 | 17543 | 1.00 | 6.5 | 0.53 |
| nested_orders | 100 | pretty | 52068 | claude-haiku-5-5 | json (persisted) | json (persisted) | 17543 | 17543 | 17543 | 1.00 | 6.5 | 0.53 |
| nested_orders | 100 | pretty | 52068 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 17543 | 17543 | 17543 | 1.00 | 6.4 | 0.50 |
| sparse_events | 20 | compact | 3005 | none | json (not_shorter) | json (not_shorter) | 1092 | 1092 | 1092 | 1.00 | 5.9 | 0.14 |
| sparse_events | 20 | compact | 3005 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 1092 | 1092 | 1092 | 1.00 | 6.6 | 0.46 |
| sparse_events | 20 | compact | 3005 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 1092 | 1092 | 1092 | 1.00 | 6.5 | 0.44 |
| sparse_events | 20 | compact | 3005 | claude-sonnet-5-5 | json (selector) | json (selector) | 1092 | 1092 | 1092 | 1.00 | 6.9 | 0.44 |
| sparse_events | 20 | pretty | 4074 | none | ldf | ldf | 1618 | 1387 | 949 | 0.59 | 5.3 | 0.15 |
| sparse_events | 20 | pretty | 4074 | claude-opus-5-5 | ldf | ldf | 1618 | 1387 | 949 | 0.59 | 6.2 | 0.47 |
| sparse_events | 20 | pretty | 4074 | claude-haiku-5-5 | ldf | ldf | 1618 | 1387 | 949 | 0.59 | 5.4 | 0.47 |
| sparse_events | 20 | pretty | 4074 | claude-sonnet-5-5 | json (selector) | json (selector) | 1618 | 1618 | 1618 | 1.00 | 6.4 | 0.42 |
| sparse_events | 100 | compact | 15400 | none | ldf | ldf | 5565 | 4835 | 4397 | 0.79 | 6.3 | 0.47 |
| sparse_events | 100 | compact | 15400 | claude-opus-5-5 | ldf | ldf | 5565 | 4835 | 4397 | 0.79 | 7.5 | 1.63 |
| sparse_events | 100 | compact | 15400 | claude-haiku-5-5 | ldf | ldf | 5565 | 4835 | 4397 | 0.79 | 7.8 | 1.64 |
| sparse_events | 100 | compact | 15400 | claude-sonnet-5-5 | json (selector) | json (selector) | 5565 | 5565 | 5565 | 1.00 | 7.3 | 1.55 |
| sparse_events | 100 | pretty | 20927 | none | ldf | ldf | 8282 | 4835 | 4397 | 0.53 | 5.5 | 0.48 |
| sparse_events | 100 | pretty | 20927 | claude-opus-5-5 | ldf | ldf | 8282 | 4835 | 4397 | 0.53 | 6.7 | 1.62 |
| sparse_events | 100 | pretty | 20927 | claude-haiku-5-5 | ldf | ldf | 8282 | 4835 | 4397 | 0.53 | 6.5 | 1.60 |
| sparse_events | 100 | pretty | 20927 | claude-sonnet-5-5 | json (selector) | json (selector) | 8282 | 8282 | 8282 | 1.00 | 6.4 | 1.53 |
| repetitive_logs | 20 | compact | 2888 | none | json (not_shorter) | json (not_shorter) | 918 | 918 | 918 | 1.00 | 4.8 | 0.10 |
| repetitive_logs | 20 | compact | 2888 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 918 | 918 | 918 | 1.00 | 5.3 | 0.32 |
| repetitive_logs | 20 | compact | 2888 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 918 | 918 | 918 | 1.00 | 5.1 | 0.30 |
| repetitive_logs | 20 | compact | 2888 | claude-sonnet-5-5 | json (selector) | json (selector) | 918 | 918 | 918 | 1.00 | 5.2 | 0.29 |
| repetitive_logs | 20 | pretty | 3849 | none | ldf | ldf | 1418 | 1094 | 656 | 0.46 | 4.7 | 0.12 |
| repetitive_logs | 20 | pretty | 3849 | claude-opus-5-5 | ldf | ldf | 1418 | 1094 | 656 | 0.46 | 5.0 | 0.32 |
| repetitive_logs | 20 | pretty | 3849 | claude-haiku-5-5 | ldf | ldf | 1418 | 1094 | 656 | 0.46 | 5.3 | 0.33 |
| repetitive_logs | 20 | pretty | 3849 | claude-sonnet-5-5 | json (selector) | json (selector) | 1418 | 1418 | 1418 | 1.00 | 5.1 | 0.29 |
| repetitive_logs | 100 | compact | 14552 | none | ldf | ldf | 4622 | 3492 | 3054 | 0.66 | 5.0 | 0.28 |
| repetitive_logs | 100 | compact | 14552 | claude-opus-5-5 | ldf | ldf | 4622 | 3492 | 3054 | 0.66 | 6.3 | 0.93 |
| repetitive_logs | 100 | compact | 14552 | claude-haiku-5-5 | ldf | ldf | 4622 | 3492 | 3054 | 0.66 | 6.2 | 0.94 |
| repetitive_logs | 100 | compact | 14552 | claude-sonnet-5-5 | json (selector) | json (selector) | 4622 | 4622 | 4622 | 1.00 | 6.1 | 0.89 |
| repetitive_logs | 100 | pretty | 19353 | none | ldf | ldf | 7122 | 3492 | 3054 | 0.43 | 5.2 | 0.33 |
| repetitive_logs | 100 | pretty | 19353 | claude-opus-5-5 | ldf | ldf | 7122 | 3492 | 3054 | 0.43 | 6.0 | 0.97 |
| repetitive_logs | 100 | pretty | 19353 | claude-haiku-5-5 | ldf | ldf | 7122 | 3492 | 3054 | 0.43 | 6.0 | 0.96 |
| repetitive_logs | 100 | pretty | 19353 | claude-sonnet-5-5 | json (selector) | json (selector) | 7122 | 7122 | 7122 | 1.00 | 5.7 | 0.91 |
| wide_table | 20 | compact | 10769 | none | ldf | ldf | 4686 | 2627 | 2189 | 0.47 | 5.1 | 0.25 |
| wide_table | 20 | compact | 10769 | claude-opus-5-5 | ldf | ldf | 4686 | 2627 | 2189 | 0.47 | 5.8 | 0.92 |
| wide_table | 20 | compact | 10769 | claude-haiku-5-5 | ldf | ldf | 4686 | 2627 | 2189 | 0.47 | 6.1 | 0.92 |
| wide_table | 20 | compact | 10769 | claude-sonnet-5-5 | json (selector) | json (selector) | 4686 | 4686 | 4686 | 1.00 | 5.8 | 0.88 |
| wide_table | 20 | pretty | 15810 | none | ldf | ldf | 7026 | 2627 | 2189 | 0.31 | 4.8 | 0.29 |
| wide_table | 20 | pretty | 15810 | claude-opus-5-5 | ldf | ldf | 7026 | 2627 | 2189 | 0.31 | 5.9 | 0.97 |
| wide_table | 20 | pretty | 15810 | claude-haiku-5-5 | ldf | ldf | 7026 | 2627 | 2189 | 0.31 | 6.0 | 0.98 |
| wide_table | 20 | pretty | 15810 | claude-sonnet-5-5 | json (selector) | json (selector) | 7026 | 7026 | 7026 | 1.00 | 5.8 | 0.94 |
| wide_table | 100 | compact | 53851 | none | json (persisted) | json (persisted) | 23421 | 23421 | 23421 | 1.00 | 5.2 | 0.51 |
| wide_table | 100 | compact | 53851 | claude-opus-5-5 | json (persisted) | json (persisted) | 23421 | 23421 | 23421 | 1.00 | 5.5 | 0.52 |
| wide_table | 100 | compact | 53851 | claude-haiku-5-5 | json (persisted) | json (persisted) | 23421 | 23421 | 23421 | 1.00 | 6.6 | 0.56 |
| wide_table | 100 | compact | 53851 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 23421 | 23421 | 23421 | 1.00 | 6.1 | 0.53 |
| wide_table | 100 | pretty | 79052 | none | json (persisted) | json (persisted) | 35121 | 35121 | 35121 | 1.00 | 5.4 | 0.76 |
| wide_table | 100 | pretty | 79052 | claude-opus-5-5 | json (persisted) | json (persisted) | 35121 | 35121 | 35121 | 1.00 | 6.0 | 0.79 |
| wide_table | 100 | pretty | 79052 | claude-haiku-5-5 | json (persisted) | json (persisted) | 35121 | 35121 | 35121 | 1.00 | 5.6 | 0.75 |
| wide_table | 100 | pretty | 79052 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 35121 | 35121 | 35121 | 1.00 | 5.5 | 0.75 |
| deep_nesting | 20 | compact | 6341 | none | ldf | ldf | 2464 | 2495 | 2057 | 0.83 | 4.8 | 0.18 |
| deep_nesting | 20 | compact | 6341 | claude-opus-5-5 | json (selector) | json (selector) | 2464 | 2464 | 2464 | 1.00 | 5.8 | 0.69 |
| deep_nesting | 20 | compact | 6341 | claude-haiku-5-5 | json (selector) | json (selector) | 2464 | 2464 | 2464 | 1.00 | 6.7 | 0.70 |
| deep_nesting | 20 | compact | 6341 | claude-sonnet-5-5 | json (selector) | json (selector) | 2464 | 2464 | 2464 | 1.00 | 5.7 | 0.70 |
| deep_nesting | 20 | pretty | 26611 | none | ldf | ldf | 6025 | 2495 | 2057 | 0.34 | 5.1 | 0.34 |
| deep_nesting | 20 | pretty | 26611 | claude-opus-5-5 | json (selector) | json (selector) | 6025 | 6025 | 6025 | 1.00 | 6.3 | 0.91 |
| deep_nesting | 20 | pretty | 26611 | claude-haiku-5-5 | json (selector) | json (selector) | 6025 | 6025 | 6025 | 1.00 | 6.6 | 0.89 |
| deep_nesting | 20 | pretty | 26611 | claude-sonnet-5-5 | json (selector) | json (selector) | 6025 | 6025 | 6025 | 1.00 | 5.7 | 0.84 |
| deep_nesting | 100 | compact | 32579 | none | json (persisted) | json (persisted) | 12560 | 12560 | 12560 | 1.00 | 4.8 | 0.26 |
| deep_nesting | 100 | compact | 32579 | claude-opus-5-5 | json (persisted) | json (persisted) | 12560 | 12560 | 12560 | 1.00 | 5.1 | 0.26 |
| deep_nesting | 100 | compact | 32579 | claude-haiku-5-5 | json (persisted) | json (persisted) | 12560 | 12560 | 12560 | 1.00 | 5.0 | 0.27 |
| deep_nesting | 100 | compact | 32579 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 12560 | 12560 | 12560 | 1.00 | 5.0 | 0.26 |
| deep_nesting | 100 | pretty | 137417 | none | json (persisted) | json (persisted) | 30673 | 30673 | 30673 | 1.00 | 5.8 | 1.15 |
| deep_nesting | 100 | pretty | 137417 | claude-opus-5-5 | json (persisted) | json (persisted) | 30673 | 30673 | 30673 | 1.00 | 6.2 | 1.17 |
| deep_nesting | 100 | pretty | 137417 | claude-haiku-5-5 | json (persisted) | json (persisted) | 30673 | 30673 | 30673 | 1.00 | 6.1 | 1.15 |
| deep_nesting | 100 | pretty | 137417 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 30673 | 30673 | 30673 | 1.00 | 6.0 | 1.15 |
| mixed_types | 20 | compact | 1426 | none | json (not_shorter) | json (not_shorter) | 492 | 492 | 492 | 1.00 | 4.8 | 0.10 |
| mixed_types | 20 | compact | 1426 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 492 | 492 | 492 | 1.00 | 5.1 | 0.31 |
| mixed_types | 20 | compact | 1426 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 492 | 492 | 492 | 1.00 | 5.4 | 0.32 |
| mixed_types | 20 | compact | 1426 | claude-sonnet-5-5 | json (selector) | json (selector) | 492 | 492 | 492 | 1.00 | 5.4 | 0.30 |
| mixed_types | 20 | pretty | 2709 | none | json (not_shorter) | json (not_shorter) | 991 | 991 | 991 | 1.00 | 5.4 | 0.11 |
| mixed_types | 20 | pretty | 2709 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 991 | 991 | 991 | 1.00 | 5.4 | 0.33 |
| mixed_types | 20 | pretty | 2709 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 991 | 991 | 991 | 1.00 | 6.1 | 0.33 |
| mixed_types | 20 | pretty | 2709 | claude-sonnet-5-5 | json (selector) | json (selector) | 991 | 991 | 991 | 1.00 | 5.3 | 0.31 |
| mixed_types | 100 | compact | 7278 | none | ldf | ldf | 2425 | 2123 | 1685 | 0.69 | 5.4 | 0.23 |
| mixed_types | 100 | compact | 7278 | claude-opus-5-5 | ldf | ldf | 2425 | 2123 | 1685 | 0.69 | 6.8 | 0.91 |
| mixed_types | 100 | compact | 7278 | claude-haiku-5-5 | ldf | ldf | 2425 | 2123 | 1685 | 0.69 | 5.6 | 0.88 |
| mixed_types | 100 | compact | 7278 | claude-sonnet-5-5 | json (selector) | json (selector) | 2425 | 2425 | 2425 | 1.00 | 6.0 | 0.86 |
| mixed_types | 100 | pretty | 13565 | none | ldf | ldf | 4853 | 2123 | 1685 | 0.35 | 5.2 | 0.26 |
| mixed_types | 100 | pretty | 13565 | claude-opus-5-5 | ldf | ldf | 4853 | 2123 | 1685 | 0.35 | 5.8 | 0.95 |
| mixed_types | 100 | pretty | 13565 | claude-haiku-5-5 | ldf | ldf | 4853 | 2123 | 1685 | 0.35 | 5.8 | 0.94 |
| mixed_types | 100 | pretty | 13565 | claude-sonnet-5-5 | json (selector) | json (selector) | 4853 | 4853 | 4853 | 1.00 | 6.6 | 0.92 |
| empty_values | 20 | compact | 3870 | none | ldf | ldf | 1191 | 968 | 530 | 0.45 | 4.7 | 0.14 |
| empty_values | 20 | compact | 3870 | claude-opus-5-5 | json (selector) | json (selector) | 1191 | 1191 | 1191 | 1.00 | 5.1 | 0.40 |
| empty_values | 20 | compact | 3870 | claude-haiku-5-5 | json (selector) | json (selector) | 1191 | 1191 | 1191 | 1.00 | 5.3 | 0.40 |
| empty_values | 20 | compact | 3870 | claude-sonnet-5-5 | json (selector) | json (selector) | 1191 | 1191 | 1191 | 1.00 | 5.2 | 0.39 |
| empty_values | 20 | pretty | 5731 | none | ldf | ldf | 1931 | 968 | 530 | 0.27 | 4.8 | 0.16 |
| empty_values | 20 | pretty | 5731 | claude-opus-5-5 | json (selector) | json (selector) | 1931 | 1931 | 1931 | 1.00 | 5.2 | 0.40 |
| empty_values | 20 | pretty | 5731 | claude-haiku-5-5 | json (selector) | json (selector) | 1931 | 1931 | 1931 | 1.00 | 5.3 | 0.40 |
| empty_values | 20 | pretty | 5731 | claude-sonnet-5-5 | json (selector) | json (selector) | 1931 | 1931 | 1931 | 1.00 | 5.5 | 0.40 |
| empty_values | 100 | compact | 19460 | none | ldf | ldf | 5956 | 2690 | 2252 | 0.38 | 5.0 | 0.44 |
| empty_values | 100 | compact | 19460 | claude-opus-5-5 | json (selector) | json (selector) | 5956 | 5956 | 5956 | 1.00 | 6.4 | 1.35 |
| empty_values | 100 | compact | 19460 | claude-haiku-5-5 | json (selector) | json (selector) | 5956 | 5956 | 5956 | 1.00 | 6.5 | 1.35 |
| empty_values | 100 | compact | 19460 | claude-sonnet-5-5 | json (selector) | json (selector) | 5956 | 5956 | 5956 | 1.00 | 6.1 | 1.31 |
| empty_values | 100 | pretty | 28761 | none | ldf | ldf | 9656 | 2690 | 2252 | 0.23 | 5.5 | 0.54 |
| empty_values | 100 | pretty | 28761 | claude-opus-5-5 | json (selector) | json (selector) | 9656 | 9656 | 9656 | 1.00 | 7.2 | 1.52 |
| empty_values | 100 | pretty | 28761 | claude-haiku-5-5 | json (selector) | json (selector) | 9656 | 9656 | 9656 | 1.00 | 7.1 | 1.49 |
| empty_values | 100 | pretty | 28761 | claude-sonnet-5-5 | json (selector) | json (selector) | 9656 | 9656 | 9656 | 1.00 | 6.4 | 1.42 |
| numeric_mix | 20 | compact | 4797 | none | ldf | ldf | 2017 | 1850 | 1412 | 0.70 | 4.8 | 0.15 |
| numeric_mix | 20 | compact | 4797 | claude-opus-5-5 | json (selector) | json (selector) | 2017 | 2017 | 2017 | 1.00 | 5.6 | 0.39 |
| numeric_mix | 20 | compact | 4797 | claude-haiku-5-5 | json (selector) | json (selector) | 2017 | 2017 | 2017 | 1.00 | 5.1 | 0.37 |
| numeric_mix | 20 | compact | 4797 | claude-sonnet-5-5 | json (selector) | json (selector) | 2017 | 2017 | 2017 | 1.00 | 5.2 | 0.36 |
| numeric_mix | 20 | pretty | 6478 | none | ldf | ldf | 2803 | 1850 | 1412 | 0.50 | 5.2 | 0.16 |
| numeric_mix | 20 | pretty | 6478 | claude-opus-5-5 | json (selector) | json (selector) | 2803 | 2803 | 2803 | 1.00 | 5.9 | 0.40 |
| numeric_mix | 20 | pretty | 6478 | claude-haiku-5-5 | json (selector) | json (selector) | 2803 | 2803 | 2803 | 1.00 | 5.5 | 0.39 |
| numeric_mix | 20 | pretty | 6478 | claude-sonnet-5-5 | json (selector) | json (selector) | 2803 | 2803 | 2803 | 1.00 | 5.4 | 0.38 |
| numeric_mix | 100 | compact | 23784 | none | ldf | ldf | 9999 | 7081 | 6643 | 0.66 | 5.8 | 0.48 |
| numeric_mix | 100 | compact | 23784 | claude-opus-5-5 | json (selector) | json (selector) | 9999 | 9999 | 9999 | 1.00 | 6.3 | 1.28 |
| numeric_mix | 100 | compact | 23784 | claude-haiku-5-5 | json (selector) | json (selector) | 9999 | 9999 | 9999 | 1.00 | 6.6 | 1.30 |
| numeric_mix | 100 | compact | 23784 | claude-sonnet-5-5 | json (selector) | json (selector) | 9999 | 9999 | 9999 | 1.00 | 6.2 | 1.25 |
| numeric_mix | 100 | pretty | 32185 | none | json (persisted) | json (persisted) | 13954 | 13954 | 13954 | 1.00 | 4.8 | 0.24 |
| numeric_mix | 100 | pretty | 32185 | claude-opus-5-5 | json (persisted) | json (persisted) | 13954 | 13954 | 13954 | 1.00 | 5.4 | 0.25 |
| numeric_mix | 100 | pretty | 32185 | claude-haiku-5-5 | json (persisted) | json (persisted) | 13954 | 13954 | 13954 | 1.00 | 5.8 | 0.25 |
| numeric_mix | 100 | pretty | 32185 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 13954 | 13954 | 13954 | 1.00 | 5.6 | 0.25 |
| high_card_strings | 20 | compact | 4701 | none | json (not_shorter) | json (not_shorter) | 2288 | 2288 | 2288 | 1.00 | 4.8 | 0.11 |
| high_card_strings | 20 | compact | 4701 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 2288 | 2288 | 2288 | 1.00 | 6.3 | 0.32 |
| high_card_strings | 20 | compact | 4701 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 2288 | 2288 | 2288 | 1.00 | 5.5 | 0.33 |
| high_card_strings | 20 | compact | 4701 | claude-sonnet-5-5 | json (selector) | json (selector) | 2288 | 2288 | 2288 | 1.00 | 6.0 | 0.30 |
| high_card_strings | 20 | pretty | 5542 | none | ldf | ldf | 2727 | 2517 | 2079 | 0.76 | 5.1 | 0.13 |
| high_card_strings | 20 | pretty | 5542 | claude-opus-5-5 | ldf | ldf | 2727 | 2517 | 2079 | 0.76 | 5.7 | 0.35 |
| high_card_strings | 20 | pretty | 5542 | claude-haiku-5-5 | ldf | ldf | 2727 | 2517 | 2079 | 0.76 | 5.2 | 0.34 |
| high_card_strings | 20 | pretty | 5542 | claude-sonnet-5-5 | json (selector) | json (selector) | 2727 | 2727 | 2727 | 1.00 | 6.2 | 0.31 |
| high_card_strings | 100 | compact | 23556 | none | ldf | ldf | 11450 | 10607 | 10169 | 0.89 | 5.0 | 0.36 |
| high_card_strings | 100 | compact | 23556 | claude-opus-5-5 | ldf | ldf | 11450 | 10607 | 10169 | 0.89 | 5.7 | 0.97 |
| high_card_strings | 100 | compact | 23556 | claude-haiku-5-5 | ldf | ldf | 11450 | 10607 | 10169 | 0.89 | 5.8 | 0.98 |
| high_card_strings | 100 | compact | 23556 | claude-sonnet-5-5 | json (selector) | json (selector) | 11450 | 11450 | 11450 | 1.00 | 5.7 | 0.92 |
| high_card_strings | 100 | pretty | 27757 | none | ldf | ldf | 13649 | 10607 | 10169 | 0.75 | 5.0 | 0.40 |
| high_card_strings | 100 | pretty | 27757 | claude-opus-5-5 | ldf | ldf | 13649 | 10607 | 10169 | 0.75 | 6.7 | 1.11 |
| high_card_strings | 100 | pretty | 27757 | claude-haiku-5-5 | ldf | ldf | 13649 | 10607 | 10169 | 0.75 | 6.0 | 1.05 |
| high_card_strings | 100 | pretty | 27757 | claude-sonnet-5-5 | json (selector) | json (selector) | 13649 | 13649 | 13649 | 1.00 | 6.9 | 1.04 |
| unicode_strings | 20 | compact | 2547 | none | json (not_shorter) | json (not_shorter) | 845 | 845 | 845 | 1.00 | 5.1 | 0.13 |
| unicode_strings | 20 | compact | 2547 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 845 | 845 | 845 | 1.00 | 5.2 | 0.35 |
| unicode_strings | 20 | compact | 2547 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 845 | 845 | 845 | 1.00 | 5.1 | 0.33 |
| unicode_strings | 20 | compact | 2547 | claude-sonnet-5-5 | json (selector) | json (selector) | 845 | 845 | 845 | 1.00 | 4.8 | 0.32 |
| unicode_strings | 20 | pretty | 3508 | none | ldf | ldf | 1297 | 982 | 544 | 0.42 | 4.4 | 0.13 |
| unicode_strings | 20 | pretty | 3508 | claude-opus-5-5 | ldf | ldf | 1297 | 982 | 544 | 0.42 | 4.8 | 0.35 |
| unicode_strings | 20 | pretty | 3508 | claude-haiku-5-5 | ldf | ldf | 1297 | 982 | 544 | 0.42 | 4.8 | 0.34 |
| unicode_strings | 20 | pretty | 3508 | claude-sonnet-5-5 | json (selector) | json (selector) | 1297 | 1297 | 1297 | 1.00 | 4.6 | 0.31 |
| unicode_strings | 100 | compact | 12950 | none | ldf | ldf | 4268 | 2902 | 2464 | 0.58 | 4.6 | 0.33 |
| unicode_strings | 100 | compact | 12950 | claude-opus-5-5 | ldf | ldf | 4268 | 2902 | 2464 | 0.58 | 5.8 | 1.07 |
| unicode_strings | 100 | compact | 12950 | claude-haiku-5-5 | ldf | ldf | 4268 | 2902 | 2464 | 0.58 | 5.5 | 1.04 |
| unicode_strings | 100 | compact | 12950 | claude-sonnet-5-5 | json (selector) | json (selector) | 4268 | 4268 | 4268 | 1.00 | 5.4 | 1.00 |
| unicode_strings | 100 | pretty | 17751 | none | ldf | ldf | 6516 | 2902 | 2464 | 0.38 | 4.6 | 0.38 |
| unicode_strings | 100 | pretty | 17751 | claude-opus-5-5 | ldf | ldf | 6516 | 2902 | 2464 | 0.38 | 5.8 | 1.12 |
| unicode_strings | 100 | pretty | 17751 | claude-haiku-5-5 | ldf | ldf | 6516 | 2902 | 2464 | 0.38 | 5.6 | 1.11 |
| unicode_strings | 100 | pretty | 17751 | claude-sonnet-5-5 | json (selector) | json (selector) | 6516 | 6516 | 6516 | 1.00 | 6.0 | 1.10 |
| long_text | 20 | compact | 24101 | none | json (not_shorter) | json (not_shorter) | 4389 | 4389 | 4389 | 1.00 | 4.6 | 0.22 |
| long_text | 20 | compact | 24101 | claude-opus-5-5 | json (selector) | json (selector) | 4389 | 4389 | 4389 | 1.00 | 5.0 | 0.58 |
| long_text | 20 | compact | 24101 | claude-haiku-5-5 | json (selector) | json (selector) | 4389 | 4389 | 4389 | 1.00 | 4.9 | 0.56 |
| long_text | 20 | compact | 24101 | claude-sonnet-5-5 | json (selector) | json (selector) | 4389 | 4389 | 4389 | 1.00 | 5.2 | 0.56 |
| long_text | 20 | pretty | 24702 | none | json (not_shorter) | json (not_shorter) | 4686 | 4686 | 4686 | 1.00 | 4.5 | 0.22 |
| long_text | 20 | pretty | 24702 | claude-opus-5-5 | json (not_shorter) | json (not_shorter) | 4686 | 4686 | 4686 | 1.00 | 5.0 | 0.59 |
| long_text | 20 | pretty | 24702 | claude-haiku-5-5 | json (not_shorter) | json (not_shorter) | 4686 | 4686 | 4686 | 1.00 | 5.0 | 0.59 |
| long_text | 20 | pretty | 24702 | claude-sonnet-5-5 | json (selector) | json (selector) | 4686 | 4686 | 4686 | 1.00 | 5.0 | 0.58 |
| long_text | 100 | compact | 123420 | none | json (persisted) | json (persisted) | 22270 | 22270 | 22270 | 1.00 | 5.2 | 0.85 |
| long_text | 100 | compact | 123420 | claude-opus-5-5 | json (persisted) | json (persisted) | 22270 | 22270 | 22270 | 1.00 | 5.2 | 0.85 |
| long_text | 100 | compact | 123420 | claude-haiku-5-5 | json (persisted) | json (persisted) | 22270 | 22270 | 22270 | 1.00 | 5.2 | 0.84 |
| long_text | 100 | compact | 123420 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 22270 | 22270 | 22270 | 1.00 | 5.2 | 0.84 |
| long_text | 100 | pretty | 126421 | none | json (persisted) | json (persisted) | 23762 | 23762 | 23762 | 1.00 | 5.1 | 0.88 |
| long_text | 100 | pretty | 126421 | claude-opus-5-5 | json (persisted) | json (persisted) | 23762 | 23762 | 23762 | 1.00 | 5.4 | 0.89 |
| long_text | 100 | pretty | 126421 | claude-haiku-5-5 | json (persisted) | json (persisted) | 23762 | 23762 | 23762 | 1.00 | 5.4 | 0.88 |
| long_text | 100 | pretty | 126421 | claude-sonnet-5-5 | json (persisted) | json (persisted) | 23762 | 23762 | 23762 | 1.00 | 5.4 | 0.88 |
