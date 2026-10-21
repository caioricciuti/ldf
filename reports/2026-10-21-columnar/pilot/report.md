# Database questions through Claude Code, claude-sonnet-5-5

Accuracy is strict (the trailing {"answer": ...} must equal the ground truth; numbers exactly, lists in order). Prompt tokens are the sum over the run's API requests of input plus cache tokens, as Anthropic reported them. Tool result chars are the text of every tool result in the transcript.

| arm | runs | correct | prompt tokens (mean) | tool result chars (mean) | turns (mean) | cost (sum) |
|---|---:|---:|---:|---:|---:|---:|
| json | 6 | 6 | 19922 | 2420 | 3.3 | $0.15 |
| minified | 6 | 6 | 19840 | 2026 | 3.3 | $0.15 |
| ldf | 6 | 6 | 20508 | 1688 | 3.7 | $0.15 |

| category | json | minified | ldf |
|---|---:|---:|---:|
| exact | 1/1 | 1/1 | 1/1 |
| filter | 1/1 | 1/1 | 1/1 |
| join | 1/1 | 1/1 | 1/1 |
| nested | 1/1 | 1/1 | 1/1 |
| null | 1/1 | 1/1 | 1/1 |
| schema | 1/1 | 1/1 | 1/1 |

- ldf vs json: 6 paired runs, +0 / -0, sign test p=1.00
- ldf vs minified: 6 paired runs, +0 / -0, sign test p=1.00
