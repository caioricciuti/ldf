# Parquet to LDF: Value path vs direct Arrow path

         Apple M2 Pro (10 logical CPUs), Darwin 27.0.0, profile release, commit 2291f06. 5 iterations per path, each in a fresh process; the time is the median, file on disk to LDF text in memory, page cache warm after the first run. Reader batch 1024 rows, block 1024 rows. `same` compares an FNV-1a hash of the output with the Value path of the same form (batch or block).

         | dataset | rows | LDF | path | median | rows/s | LDF MB/s | peak RSS | vs Value | same |
         |---|---:|---:|---|---:|---:|---:|---:|---:|---|
| deep_nesting | 2000 | 648.2 KiB | value-batch | 6.7 ms | 0.30 M | 100 | 24.0 MiB | 1.00x | base |
| deep_nesting | 2000 | 648.2 KiB | direct-batch | 4.8 ms | 0.41 M | 137 | 19.3 MiB | 1.38x | yes |
| deep_nesting | 2000 | 648.2 KiB | value-block | 6.5 ms | 0.31 M | 103 | 19.9 MiB | 1.00x | base |
| deep_nesting | 2000 | 648.2 KiB | direct-block | 4.6 ms | 0.43 M | 143 | 15.6 MiB | 1.40x | yes |
| flat_users | 10000 | 632.3 KiB | value-batch | 6.9 ms | 1.45 M | 94 | 17.0 MiB | 1.00x | base |
| flat_users | 10000 | 632.3 KiB | direct-batch | 2.6 ms | 3.90 M | 253 | 11.7 MiB | 2.70x | yes |
| flat_users | 10000 | 632.3 KiB | value-block | 6.9 ms | 1.45 M | 94 | 11.2 MiB | 1.00x | base |
| flat_users | 10000 | 632.3 KiB | direct-block | 2.8 ms | 3.61 M | 234 | 11.6 MiB | 2.49x | yes |
| high_card_strings | 10000 | 1.8 MiB | value-batch | 9.1 ms | 1.10 M | 203 | 25.0 MiB | 1.00x | base |
| high_card_strings | 10000 | 1.8 MiB | direct-batch | 4.9 ms | 2.03 M | 375 | 19.0 MiB | 1.85x | yes |
| high_card_strings | 10000 | 1.8 MiB | value-block | 8.6 ms | 1.17 M | 215 | 13.5 MiB | 1.00x | base |
| high_card_strings | 10000 | 1.8 MiB | direct-block | 4.8 ms | 2.10 M | 387 | 12.2 MiB | 1.80x | yes |
| long_text | 2000 | 2.3 MiB | value-batch | 4.8 ms | 0.42 M | 494 | 19.2 MiB | 1.00x | base |
| long_text | 2000 | 2.3 MiB | direct-batch | 4.1 ms | 0.49 M | 582 | 17.8 MiB | 1.18x | yes |
| long_text | 2000 | 2.3 MiB | value-block | 4.4 ms | 0.45 M | 536 | 16.6 MiB | 1.00x | base |
| long_text | 2000 | 2.3 MiB | direct-block | 3.9 ms | 0.51 M | 601 | 16.3 MiB | 1.12x | yes |
| nested_orders | 10000 | 1.1 MiB | value-batch | 19.8 ms | 0.50 M | 61 | 32.1 MiB | 1.00x | base |
| nested_orders | | | direct-batch | | | | | | not supported by the direct Arrow path: `items` is a list of structs (a child table) |
| nested_orders | 10000 | 1.2 MiB | value-block | 18.9 ms | 0.53 M | 64 | 14.2 MiB | 1.00x | base |
| nested_orders | | | direct-block | | | | | | not supported by the direct Arrow path: `items` is a list of structs (a child table) |
| numeric_mix | 10000 | 1.3 MiB | value-batch | 11.7 ms | 0.86 M | 118 | 21.7 MiB | 1.00x | base |
| numeric_mix | 10000 | 1.3 MiB | direct-batch | 5.7 ms | 1.74 M | 240 | 11.1 MiB | 2.03x | yes |
| numeric_mix | 10000 | 1.3 MiB | value-block | 10.8 ms | 0.93 M | 128 | 12.0 MiB | 1.00x | base |
| numeric_mix | 10000 | 1.3 MiB | direct-block | 5.7 ms | 1.77 M | 243 | 10.6 MiB | 1.90x | yes |
| repetitive_logs | 10000 | 676.8 KiB | value-batch | 6.4 ms | 1.56 M | 108 | 17.7 MiB | 1.00x | base |
| repetitive_logs | 10000 | 676.8 KiB | direct-batch | 2.1 ms | 4.67 M | 324 | 11.2 MiB | 3.00x | yes |
| repetitive_logs | 10000 | 676.8 KiB | value-block | 6.3 ms | 1.59 M | 110 | 10.8 MiB | 1.00x | base |
| repetitive_logs | 10000 | 676.8 KiB | direct-block | 2.1 ms | 4.70 M | 326 | 9.8 MiB | 2.95x | yes |
| sparse_events | 10000 | 2.0 MiB | value-batch | 15.9 ms | 0.63 M | 130 | 26.4 MiB | 1.00x | base |
| sparse_events | 10000 | 2.0 MiB | direct-batch | 7.1 ms | 1.41 M | 291 | 14.4 MiB | 2.24x | yes |
| sparse_events | 10000 | 2.0 MiB | value-block | 15.4 ms | 0.65 M | 136 | 12.8 MiB | 1.00x | base |
| sparse_events | 10000 | 2.0 MiB | direct-block | 7.1 ms | 1.41 M | 295 | 11.2 MiB | 2.16x | yes |
| unicode_strings | 10000 | 636.7 KiB | value-batch | 7.2 ms | 1.38 M | 90 | 16.7 MiB | 1.00x | base |
| unicode_strings | 10000 | 636.7 KiB | direct-batch | 3.1 ms | 3.27 M | 213 | 10.7 MiB | 2.37x | yes |
| unicode_strings | 10000 | 636.7 KiB | value-block | 7.3 ms | 1.37 M | 89 | 10.7 MiB | 1.00x | base |
| unicode_strings | 10000 | 636.7 KiB | direct-block | 3.1 ms | 3.19 M | 208 | 9.8 MiB | 2.33x | yes |
| wide_table | 10000 | 2.6 MiB | value-batch | 42.7 ms | 0.23 M | 64 | 54.7 MiB | 1.00x | base |
| wide_table | 10000 | 2.6 MiB | direct-batch | 14.2 ms | 0.70 M | 194 | 23.5 MiB | 3.01x | yes |
| wide_table | 10000 | 2.6 MiB | value-block | 33.3 ms | 0.30 M | 83 | 21.6 MiB | 1.00x | base |
| wide_table | 10000 | 2.6 MiB | direct-block | 14.0 ms | 0.71 M | 196 | 18.2 MiB | 2.38x | yes |
