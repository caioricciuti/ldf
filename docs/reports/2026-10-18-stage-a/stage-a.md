# Stage A: token search over the knob space

Fixtures `medium`, first 200 records per dataset, tokenizer `o200k_base` (exact, tiktoken-rs). 154440 configurations, encoded once each; distinct outputs round-trip checked and counted. A candidate passes when it is lossless (byte, semantic or semantic-unordered) and at most 0.90 of compact JSON tokens. Run took 66 s on 10 threads. Token counts do not depend on the machine.

## Room beyond LDF v0.1

| dataset | json | ldf v0.1 | v0.1 / json | best | best / json | best / v0.1 | distinct | lossless | passing | engine v0.1 = crates/ldf |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| deep_nesting | 25218 | 20360 | 0.807 | 20352 | 0.807 | 1.000 | 36 | 36 | 12 | yes |
| empty_values | 11920 | 4358 | 0.366 | 4198 | 0.352 | 0.963 | 1428 | 1428 | 1176 | yes |
| flat_users | 9376 | 5702 | 0.608 | 5671 | 0.605 | 0.995 | 63 | 63 | 47 | yes |
| high_card_strings | 22863 | 20219 | 0.884 | 20137 | 0.881 | 0.996 | 18 | 18 | 12 | yes |
| long_text | 43862 | 41947 | 0.956 | - | - | - | 54 | 54 | 0 | yes |
| mixed_types | 4917 | 3566 | 0.725 | 3284 | 0.668 | 0.921 | 486 | 486 | 468 | yes |
| nested_orders | 19980 | 12888 | 0.645 | 10931 | 0.547 | 0.848 | 6642 | 6642 | 5913 | yes |
| numeric_mix | 19968 | 13129 | 0.658 | 13077 | 0.655 | 0.996 | 63 | 63 | 42 | yes |
| repetitive_logs | 9226 | 5974 | 0.648 | 5644 | 0.612 | 0.945 | 18 | 18 | 12 | yes |
| sparse_events | 11050 | 8578 | 0.776 | 6835 | 0.619 | 0.797 | 2466 | 2466 | 2310 | yes |
| unicode_strings | 8431 | 4693 | 0.557 | 4638 | 0.550 | 0.988 | 54 | 54 | 48 | yes |
| wide_table | 46836 | 20250 | 0.432 | 19810 | 0.423 | 0.978 | 126 | 126 | 112 | yes |

## Best candidate per dataset

**deep_nesting**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 20352 | 0.807 | 1.000 | byte | 1296 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 2 | 20360 | 0.807 | 1.000 | byte | 15552 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 3 | 20447 | 0.811 | 1.004 | byte | 1296 | `d=pipe h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 4 | 20448 | 0.811 | 1.004 | byte | 648 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=every50 fl=keep b=words tr=0` |
| 5 | 20455 | 0.811 | 1.005 | byte | 15552 | `d=pipe h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |

**empty_values**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 4198 | 0.352 | 0.963 | byte | 224 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=empty/? qt=json k=once fl=keep b=words tr=0` |
| 2 | 4216 | 0.354 | 0.967 | byte | 448 | `d=comma h=typed q?=0 o=dotted a=child-key sp=0 rf=json n=empty/? qt=json k=once fl=keep b=letters tr=0` |
| 3 | 4216 | 0.354 | 0.967 | byte | 448 | `d=comma h=typed q?=0 o=dotted a=child-key sp=0 rf=json n=empty/? qt=json k=once fl=keep b=words tr=0` |
| 4 | 4217 | 0.354 | 0.968 | byte | 448 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=empty/? qt=json k=once fl=keep b=letters tr=0` |
| 5 | 4217 | 0.354 | 0.968 | byte | 448 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=empty/? qt=json k=once fl=keep b=words tr=0` |

**flat_users**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 5671 | 0.605 | 0.995 | byte | 2592 | `d=tab h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 2 | 5675 | 0.605 | 0.995 | byte | 5184 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=typed b=letters tr=0` |
| 3 | 5675 | 0.605 | 0.995 | byte | 5184 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=typed b=words tr=0` |
| 4 | 5679 | 0.606 | 0.996 | byte | 5184 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=letters tr=0` |
| 5 | 5679 | 0.606 | 0.996 | byte | 5184 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |

**high_card_strings**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 20137 | 0.881 | 0.996 | byte | 2592 | `d=tab h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 2 | 20143 | 0.881 | 0.996 | byte | 31104 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 3 | 20167 | 0.882 | 0.997 | byte | 1296 | `d=tab h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=every50 fl=keep b=words tr=0` |
| 4 | 20191 | 0.883 | 0.999 | byte | 15552 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=every50 fl=keep b=words tr=0` |
| 5 | 20213 | 0.884 | 1.000 | byte | 2592 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |

**long_text**

No lossless candidate within the bar.

**mixed_types**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 3284 | 0.668 | 0.921 | byte | 192 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=empty/? qt=json k=once fl=keep b=words tr=0` |
| 2 | 3285 | 0.668 | 0.921 | byte | 96 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=empty/? qt=backslash k=once fl=keep b=words tr=0` |
| 3 | 3303 | 0.672 | 0.926 | byte | 192 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=dash/empty qt=json k=once fl=keep b=words tr=0` |
| 4 | 3305 | 0.672 | 0.927 | byte | 96 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=dash/empty qt=backslash k=once fl=keep b=words tr=0` |
| 5 | 3311 | 0.673 | 0.928 | byte | 96 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=empty/? qt=json k=every50 fl=keep b=words tr=0` |

**nested_orders**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 10931 | 0.547 | 0.848 | byte | 40 | `d=comma h=untyped q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=json k=once fl=keep b=words tr=0` |
| 2 | 10931 | 0.547 | 0.848 | byte | 40 | `d=comma h=untyped q?=0 o=dotted a=hoisted sp=0 rf=json n=empty/? qt=json k=once fl=keep b=words tr=0` |
| 3 | 10933 | 0.547 | 0.848 | byte | 60 | `d=comma h=typed q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=json k=once fl=typed b=letters tr=0` |
| 4 | 10933 | 0.547 | 0.848 | byte | 60 | `d=comma h=typed q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=json k=once fl=typed b=words tr=0` |
| 5 | 10933 | 0.547 | 0.848 | byte | 60 | `d=comma h=typed q?=0 o=dotted a=hoisted sp=0 rf=json n=empty/? qt=json k=once fl=typed b=letters tr=0` |

**numeric_mix**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 13077 | 0.655 | 0.996 | byte | 5184 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=typed b=letters tr=0` |
| 2 | 13077 | 0.655 | 0.996 | byte | 5184 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=typed b=words tr=0` |
| 3 | 13116 | 0.657 | 0.999 | byte | 2592 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 4 | 13129 | 0.658 | 1.000 | byte | 5184 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=letters tr=0` |
| 5 | 13129 | 0.658 | 1.000 | byte | 5184 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |

**repetitive_logs**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 5644 | 0.612 | 0.945 | byte | 2592 | `d=tab h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 2 | 5651 | 0.613 | 0.946 | byte | 31104 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 3 | 5686 | 0.616 | 0.952 | byte | 1296 | `d=tab h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=every50 fl=keep b=words tr=0` |
| 4 | 5714 | 0.619 | 0.956 | byte | 15552 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=every50 fl=keep b=words tr=0` |
| 5 | 5967 | 0.647 | 0.999 | byte | 2592 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |

**sparse_events**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 6835 | 0.619 | 0.797 | semantic-unordered | 32 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=dash/empty qt=json k=once fl=keep b=words tr=1` |
| 2 | 6836 | 0.619 | 0.797 | semantic-unordered | 16 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=dash/empty qt=backslash k=once fl=keep b=words tr=1` |
| 3 | 6848 | 0.620 | 0.798 | semantic-unordered | 32 | `d=comma h=typed q?=0 o=dotted a=child-key sp=0 rf=json n=dash/empty qt=json k=once fl=typed b=letters tr=1` |
| 4 | 6848 | 0.620 | 0.798 | semantic-unordered | 32 | `d=comma h=typed q?=0 o=dotted a=child-key sp=0 rf=json n=dash/empty qt=json k=once fl=typed b=words tr=1` |
| 5 | 6849 | 0.620 | 0.798 | semantic-unordered | 16 | `d=comma h=typed q?=0 o=dotted a=child-key sp=0 rf=json n=dash/empty qt=backslash k=once fl=typed b=letters tr=1` |

**unicode_strings**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 4638 | 0.550 | 0.988 | byte | 10368 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=backslash k=once fl=keep b=words tr=0` |
| 2 | 4638 | 0.550 | 0.988 | byte | 10368 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=csv k=once fl=keep b=words tr=0` |
| 3 | 4662 | 0.553 | 0.993 | byte | 864 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=csv k=once fl=keep b=words tr=0` |
| 4 | 4675 | 0.555 | 0.996 | byte | 864 | `d=comma h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=backslash k=once fl=keep b=words tr=0` |
| 5 | 4693 | 0.557 | 1.000 | byte | 10368 | `d=comma h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |

**wide_table**

| rank | tokens | / json | / v0.1 | level | configs | id |
|---:|---:|---:|---:|---|---:|---|
| 1 | 19810 | 0.423 | 0.978 | byte | 864 | `d=tab h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=backslash k=once fl=keep b=words tr=0` |
| 2 | 19810 | 0.423 | 0.978 | byte | 1728 | `d=tab h=untyped q?=0 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |
| 3 | 19845 | 0.424 | 0.980 | byte | 1728 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=backslash k=once fl=typed b=letters tr=0` |
| 4 | 19845 | 0.424 | 0.980 | byte | 1728 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=backslash k=once fl=typed b=words tr=0` |
| 5 | 19845 | 0.424 | 0.980 | byte | 3456 | `d=tab h=typed q?=1 o=dotted a=child-key sp=0 rf=json n=null/empty qt=json k=once fl=typed b=letters tr=0` |

## Reference formats (tokens / compact JSON)

| dataset | json | jsonl | csv | tsv | toon | toon-tab | yaml | ldf | ldf-v0 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| deep_nesting | 1.000 | 1.008 | unsupported | unsupported | 1.810 | 1.913 | 1.865 | 0.807 | 0.807 |
| empty_values | 1.000 | 1.017 | unsupported | unsupported | 1.271 | 1.271 | 1.216 | 0.366 | 0.366 |
| flat_users | 1.000 | 1.021 | 0.607 | 0.604 | 0.650 | 0.647 | 1.247 | 0.608 | 0.608 |
| high_card_strings | 1.000 | 1.009 | 0.884 | 0.881 | 0.904 | 0.907 | 1.074 | 0.884 | 0.884 |
| long_text | 1.000 | 1.005 | 0.939 | 0.941 | 0.965 | 0.968 | 1.019 | 0.956 | 0.956 |
| mixed_types | 1.000 | 1.014 | unsupported | unsupported | 1.327 | 1.336 | 1.330 | 0.725 | 0.681 |
| nested_orders | 1.000 | 1.010 | unsupported | unsupported | 1.082 | 1.082 | 1.256 | 0.645 | 0.645 |
| numeric_mix | 1.000 | 1.010 | 0.651 | 0.665 | 0.671 | 0.685 | 1.206 | 0.658 | 0.658 |
| repetitive_logs | 1.000 | 1.021 | 0.647 | 0.612 | 0.690 | 0.677 | 1.193 | 0.648 | 0.648 |
| sparse_events | 1.000 | 1.018 | 0.612 (lossy) | 0.609 (lossy) | 1.222 | 1.222 | 1.198 | 0.776 | 0.638 |
| unicode_strings | 1.000 | 1.023 | 0.520 (lossy) | 0.551 (lossy) | 0.615 | 0.708 | 1.134 | 0.557 | 0.557 |
| wide_table | 1.000 | 1.004 | 0.428 (lossy) | 0.414 (lossy) | 0.440 | 0.431 | 1.267 | 0.432 | 0.432 |

## One configuration for all datasets

Configurations lossless on every dataset, ranked by mean tokens / JSON.

| rank | mean / json | mean / v0.1 | worst / json | id |
|---:|---:|---:|---:|---|
| 1 | 0.643 | 0.960 | 0.939 | `d=comma h=untyped q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=once fl=keep b=words tr=1` |
| 2 | 0.643 | 0.960 | 0.939 | `d=comma h=untyped q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=every200 fl=keep b=words tr=1` |
| 3 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=once fl=typed b=words tr=1` |
| 4 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=once fl=typed b=letters tr=1` |
| 5 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=every200 fl=typed b=words tr=1` |
| 6 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=every200 fl=typed b=letters tr=1` |
| 7 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=1 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=once fl=typed b=words tr=1` |
| 8 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=1 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=once fl=typed b=letters tr=1` |
| 9 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=1 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=every200 fl=typed b=words tr=1` |
| 10 | 0.644 | 0.960 | 0.939 | `d=comma h=typed q?=1 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=every200 fl=typed b=letters tr=1` |
| v0.1 | 0.672 | 1.000 | 0.956 | `d=comma h=typed q?=1 o=dotted a=child-key sp=50 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` |

## Knob effects

Per dataset: `-` when no change of the knob alone changes the output; otherwise the value with the lowest lossless tokens and, in brackets, how much more the best configuration with the worst value costs.

| knob | deep_nesting | empty_values | flat_users | high_card_strings | long_text | mixed_types | nested_orders | numeric_mix | repetitive_logs | sparse_events | unicode_strings | wide_table |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| delimiter | comma (+1.0%) | comma (+16.3%) | tab (+5.6%) | tab (+1.6%) | comma (+0.3%) | comma (+5.8%) | comma (+6.4%) | comma (+3.1%) | tab (+7.1%) | comma (+2.6%) | comma (+8.4%) | tab (+13.7%) |
| header typing | untyped (+0.0%) | untyped (+0.4%) | untyped (+0.1%) | untyped (+0.0%) | untyped (+0.0%) | untyped (+0.9%) | untyped (+0.0%) | typed (+0.3%) | untyped (+0.1%) | untyped (+0.2%) | typed (+0.5%) | untyped (+0.2%) |
| nullable mark | - | 0 (+0.5%) | - | - | - | 0 (+0.9%) | 0 (+0.0%) | - | - | 0 (+0.2%) | - | - |
| objects | dotted (+17.6%) | dotted (+28.5%) | - | - | - | - | dotted (+27.9%) | - | - | - | - | - |
| arrays of objects | - | - | - | - | - | - | hoisted (+24.4%) | - | - | - | - | - |
| sparse threshold | - | 0 (+28.8%) | - | - | - | 0 (+16.2%) | 0 (+3.7%) | - | - | 0 (+23.4%) | - | - |
| sparse cell form | - | json (+0.0%) | - | - | - | json (+0.0%) | json (+0.0%) | - | - | json (+5.9%) | - | - |
| null / absent | - | empty/? (+3.4%) | - | - | - | empty/? (+1.0%) | dash/empty (+0.5%) | - | - | dash/empty (+5.0%) | - | - |
| quoting | - | json (+0.5%) | - | - | csv (+1.8%) | json (+0.0%) | json (+0.0%) | - | - | json (+0.0%) | csv (+1.2%) | json (+0.0%) |
| key repetition | once (+22.5%) | once (+188.4%) | once (+46.9%) | once (+7.5%) | once (+3.0%) | once (+27.5%) | once (+43.9%) | once (+41.3%) | once (+48.1%) | once (+39.4%) | once (+58.0%) | once (+85.3%) |
| float kind | - | - | keep (+0.1%) | - | - | - | keep (+0.0%) | typed (+0.3%) | - | keep (+0.2%) | - | keep (+0.2%) |
| booleans | - | words (+5.2%) | words (+3.6%) | - | - | - | words (+1.8%) | words (+3.1%) | - | words (+0.5%) | - | words (+10.3%) |
| trim trailing absent | - | - | - | - | - | 0 (+0.0%) | - | - | - | 1 (+2.6%) | - | - |

## Recheck at 1000 records

The top candidates of each dataset re-encoded on the larger sample, sorted by tokens; `rank` is the rank they had at 200 records.

**deep_nesting** (1000 records, json 123455, v0.1 99049 = 0.802)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 99041 | 0.802 | 1.000 | byte |
| 2 | 99049 | 0.802 | 1.000 | byte |
| 3 | 99484 | 0.806 | 1.004 | byte |
| 5 | 99492 | 0.806 | 1.004 | byte |
| 4 | 99649 | 0.807 | 1.006 | byte |
| 6 | 99809 | 0.808 | 1.008 | byte |
| 8 | 100035 | 0.810 | 1.010 | byte |
| 9 | 100043 | 0.810 | 1.010 | byte |
| 7 | 100092 | 0.811 | 1.011 | byte |
| 10 | 100252 | 0.812 | 1.012 | byte |

**empty_values** (1000 records, json 59714, v0.1 21665 = 0.363)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 20949 | 0.351 | 0.967 | byte |
| 2 | 20967 | 0.351 | 0.968 | byte |
| 3 | 20967 | 0.351 | 0.968 | byte |
| 4 | 20968 | 0.351 | 0.968 | byte |
| 5 | 20968 | 0.351 | 0.968 | byte |
| 6 | 21063 | 0.353 | 0.972 | byte |
| 7 | 21081 | 0.353 | 0.973 | byte |
| 8 | 21081 | 0.353 | 0.973 | byte |
| 9 | 21082 | 0.353 | 0.973 | byte |
| 10 | 21082 | 0.353 | 0.973 | byte |

**flat_users** (1000 records, json 46854, v0.1 28371 = 0.606)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 2 | 28264 | 0.603 | 0.996 | byte |
| 3 | 28264 | 0.603 | 0.996 | byte |
| 1 | 28272 | 0.603 | 0.997 | byte |
| 4 | 28280 | 0.604 | 0.997 | byte |
| 5 | 28280 | 0.604 | 0.997 | byte |
| 7 | 28355 | 0.605 | 0.999 | byte |
| 8 | 28355 | 0.605 | 0.999 | byte |
| 6 | 28363 | 0.605 | 1.000 | byte |
| 9 | 28371 | 0.606 | 1.000 | byte |
| 10 | 28371 | 0.606 | 1.000 | byte |

**high_card_strings** (1000 records, json 114671, v0.1 101416 = 0.884)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 101003 | 0.881 | 0.996 | byte |
| 2 | 101009 | 0.881 | 0.996 | byte |
| 3 | 101193 | 0.882 | 0.998 | byte |
| 4 | 101313 | 0.884 | 0.999 | byte |
| 5 | 101410 | 0.884 | 1.000 | byte |
| 6 | 101416 | 0.884 | 1.000 | byte |
| 7 | 101638 | 0.886 | 1.002 | byte |
| 8 | 101758 | 0.887 | 1.003 | byte |
| 9 | 102644 | 0.895 | 1.012 | byte |
| 10 | 102650 | 0.895 | 1.012 | byte |

**long_text** (1000 records, json 224271, v0.1 214649 = 0.957)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|

**mixed_types** (1000 records, json 24896, v0.1 18118 = 0.728)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 16555 | 0.665 | 0.914 | byte |
| 2 | 16580 | 0.666 | 0.915 | byte |
| 7 | 16670 | 0.670 | 0.920 | byte |
| 8 | 16670 | 0.670 | 0.920 | byte |
| 3 | 16694 | 0.671 | 0.921 | byte |
| 4 | 16720 | 0.672 | 0.923 | byte |
| 5 | 16726 | 0.672 | 0.923 | byte |
| 9 | 16750 | 0.673 | 0.924 | byte |
| 6 | 16751 | 0.673 | 0.925 | byte |
| 10 | 16773 | 0.674 | 0.926 | byte |

**nested_orders** (1000 records, json 99948, v0.1 64238 = 0.643)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 3 | 54453 | 0.545 | 0.848 | byte |
| 4 | 54453 | 0.545 | 0.848 | byte |
| 5 | 54453 | 0.545 | 0.848 | byte |
| 6 | 54453 | 0.545 | 0.848 | byte |
| 7 | 54453 | 0.545 | 0.848 | byte |
| 8 | 54453 | 0.545 | 0.848 | byte |
| 9 | 54453 | 0.545 | 0.848 | byte |
| 10 | 54453 | 0.545 | 0.848 | byte |
| 1 | 54479 | 0.545 | 0.848 | byte |
| 2 | 54479 | 0.545 | 0.848 | byte |

**numeric_mix** (1000 records, json 99863, v0.1 65490 = 0.656)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 65272 | 0.654 | 0.997 | byte |
| 2 | 65272 | 0.654 | 0.997 | byte |
| 3 | 65477 | 0.656 | 1.000 | byte |
| 4 | 65490 | 0.656 | 1.000 | byte |
| 5 | 65490 | 0.656 | 1.000 | byte |
| 8 | 66066 | 0.662 | 1.009 | byte |
| 6 | 66108 | 0.662 | 1.009 | byte |
| 7 | 66108 | 0.662 | 1.009 | byte |
| 9 | 66326 | 0.664 | 1.013 | byte |
| 10 | 66326 | 0.664 | 1.013 | byte |

**repetitive_logs** (1000 records, json 46047, v0.1 29729 = 0.646)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 28147 | 0.611 | 0.947 | byte |
| 2 | 28154 | 0.611 | 0.947 | byte |
| 3 | 28413 | 0.617 | 0.956 | byte |
| 4 | 28553 | 0.620 | 0.960 | byte |
| 5 | 29722 | 0.645 | 1.000 | byte |
| 6 | 29729 | 0.646 | 1.000 | byte |
| 7 | 30026 | 0.652 | 1.010 | byte |
| 8 | 30063 | 0.653 | 1.011 | byte |
| 10 | 30070 | 0.653 | 1.011 | byte |
| 9 | 30166 | 0.655 | 1.015 | byte |

**sparse_events** (1000 records, json 54975, v0.1 43943 = 0.799)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 33743 | 0.614 | 0.768 | semantic-unordered |
| 3 | 33750 | 0.614 | 0.768 | semantic-unordered |
| 4 | 33750 | 0.614 | 0.768 | semantic-unordered |
| 2 | 33756 | 0.614 | 0.768 | semantic-unordered |
| 7 | 33758 | 0.614 | 0.768 | semantic-unordered |
| 8 | 33758 | 0.614 | 0.768 | semantic-unordered |
| 5 | 33763 | 0.614 | 0.768 | semantic-unordered |
| 6 | 33763 | 0.614 | 0.768 | semantic-unordered |
| 9 | 33771 | 0.614 | 0.769 | semantic-unordered |
| 10 | 33771 | 0.614 | 0.769 | semantic-unordered |

**unicode_strings** (1000 records, json 42188, v0.1 23345 = 0.553)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 2 | 23082 | 0.547 | 0.989 | byte |
| 1 | 23158 | 0.549 | 0.992 | byte |
| 3 | 23220 | 0.550 | 0.995 | byte |
| 5 | 23345 | 0.553 | 1.000 | byte |
| 4 | 23352 | 0.554 | 1.000 | byte |
| 9 | 23500 | 0.557 | 1.007 | byte |
| 7 | 23519 | 0.557 | 1.007 | byte |
| 8 | 23524 | 0.558 | 1.008 | byte |
| 6 | 23595 | 0.559 | 1.011 | byte |
| 10 | 23656 | 0.561 | 1.013 | byte |

**wide_table** (1000 records, json 234201, v0.1 100603 = 0.430)

| rank | tokens | / json | / v0.1 | level |
|---:|---:|---:|---:|---|
| 1 | 98506 | 0.421 | 0.979 | byte |
| 2 | 98506 | 0.421 | 0.979 | byte |
| 3 | 98529 | 0.421 | 0.979 | byte |
| 4 | 98529 | 0.421 | 0.979 | byte |
| 5 | 98529 | 0.421 | 0.979 | byte |
| 6 | 98529 | 0.421 | 0.979 | byte |
| 7 | 98547 | 0.421 | 0.980 | byte |
| 8 | 98547 | 0.421 | 0.980 | byte |
| 9 | 98547 | 0.421 | 0.980 | byte |
| 10 | 98547 | 0.421 | 0.980 | byte |

## Dropped outputs

| dataset | lossy | decode-error | first failure |
|---|---:|---:|---|
| deep_nesting | 0 | 0 |  |
| empty_values | 0 | 0 |  |
| flat_users | 0 | 0 |  |
| high_card_strings | 0 | 0 |  |
| long_text | 0 | 0 |  |
| mixed_types | 0 | 0 |  |
| nested_orders | 0 | 0 |  |
| numeric_mix | 0 | 0 |  |
| repetitive_logs | 0 | 0 |  |
| sparse_events | 0 | 0 |  |
| unicode_strings | 0 | 0 |  |
| wide_table | 0 | 0 |  |
