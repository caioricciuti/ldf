# Parquet to LDF: Value path vs direct Arrow path

Apple M2 Pro (10 logical CPUs), Darwin 27.0.0, profile release, commit 2291f06. 3 iterations per path, each in a fresh process; the time is the median, file on disk to LDF text in memory, page cache warm after the first run. Reader batch 1024 rows, block 1024 rows. `same` compares an FNV-1a hash of the output with the Value path of the same form (batch or block).

| dataset | rows | LDF | path | median | rows/s | LDF MB/s | peak RSS | vs Value | same |
|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| flat_users | 1000000 | 65.6 MiB | value-batch | 1072.3 ms | 0.93 M | 64 | 909.0 MiB | 1.00x | base |
| flat_users | 1000000 | 65.6 MiB | direct-batch | 240.7 ms | 4.16 M | 286 | 310.2 MiB | 4.46x | yes |
| flat_users | 1000000 | 65.6 MiB | value-block | 665.6 ms | 1.50 M | 103 | 20.8 MiB | 1.00x | base |
| flat_users | 1000000 | 65.6 MiB | direct-block | 232.0 ms | 4.31 M | 296 | 16.3 MiB | 2.87x | yes |
| high_card_strings | 500000 | 88.8 MiB | value-batch | 623.3 ms | 0.80 M | 149 | 625.6 MiB | 1.00x | base |
| high_card_strings | 500000 | 88.8 MiB | direct-batch | 225.5 ms | 2.22 M | 413 | 437.3 MiB | 2.76x | yes |
| high_card_strings | 500000 | 88.8 MiB | value-block | 401.7 ms | 1.24 M | 232 | 31.9 MiB | 1.00x | base |
| high_card_strings | 500000 | 88.8 MiB | direct-block | 215.8 ms | 2.32 M | 431 | 36.8 MiB | 1.86x | yes |
| numeric_mix | 1000000 | 132.9 MiB | value-batch | 1880.5 ms | 0.53 M | 74 | 1.4 GiB | 1.00x | base |
| numeric_mix | 1000000 | 132.9 MiB | direct-batch | 574.8 ms | 1.74 M | 242 | 507.6 MiB | 3.27x | yes |
| numeric_mix | 1000000 | 132.9 MiB | value-block | 1081.2 ms | 0.92 M | 129 | 32.9 MiB | 1.00x | base |
| numeric_mix | 1000000 | 132.9 MiB | direct-block | 559.4 ms | 1.79 M | 249 | 22.8 MiB | 1.93x | yes |
| repetitive_logs | 1000000 | 66.1 MiB | value-batch | 993.3 ms | 1.01 M | 70 | 911.0 MiB | 1.00x | base |
| repetitive_logs | 1000000 | 66.1 MiB | direct-batch | 208.4 ms | 4.80 M | 332 | 332.3 MiB | 4.77x | yes |
| repetitive_logs | 1000000 | 66.1 MiB | value-block | 630.0 ms | 1.59 M | 110 | 14.0 MiB | 1.00x | base |
| repetitive_logs | 1000000 | 66.1 MiB | direct-block | 199.8 ms | 5.01 M | 347 | 12.4 MiB | 3.15x | yes |
| sparse_events | 1000000 | 200.9 MiB | value-batch | 2285.0 ms | 0.44 M | 92 | 1.7 GiB | 1.00x | base |
| sparse_events | 1000000 | 200.9 MiB | direct-batch | 694.7 ms | 1.44 M | 303 | 775.2 MiB | 3.29x | yes |
| sparse_events | 1000000 | 199.7 MiB | value-block | 1548.7 ms | 0.65 M | 135 | 21.8 MiB | 1.00x | base |
| sparse_events | 1000000 | 199.7 MiB | direct-block | 694.2 ms | 1.44 M | 302 | 19.8 MiB | 2.23x | yes |
| wide_table | 1000000 | 264.5 MiB | value-batch | 6177.8 ms | 0.16 M | 45 | 3.8 GiB | 1.00x | base |
| wide_table | 1000000 | 264.5 MiB | direct-batch | 1424.6 ms | 0.70 M | 195 | 874.3 MiB | 4.34x | yes |
| wide_table | 1000000 | 264.5 MiB | value-block | 3335.6 ms | 0.30 M | 83 | 83.8 MiB | 1.00x | base |
| wide_table | 1000000 | 264.5 MiB | direct-block | 1384.4 ms | 0.72 M | 200 | 49.9 MiB | 2.41x | yes |
