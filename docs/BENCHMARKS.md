# Benchmarks

Every number below was measured on one machine (MacBook Pro M1 Pro, 32 GB) against
one corpus: 296 markdown notes in French and English, 21 projects, 1 661 paragraphs,
plus 4 964 generated questions, 6 625 vectors in total. The corpus is private; the
method and the tooling are in this repository so anyone can reproduce the protocol on
their own notes.

## Protocol

Four query families, reported separately and never merged into one score:

| family | cases | what it targets |
|---|---|---|
| topic of a note | 24 | the subject of a note, query written blind from its first chunk |
| buried detail | 24 | a passage in the second half of a note, query written blind from that passage only |
| named identifier | 12 | a query naming an identifier (ticket key, field, class) present in one note |
| first benchmark | 36 | the queries of the first attempt, one third in the other language than the note |

Blind means the author of the query sees the target passage and nothing else, never
the title or the description. One third of the queries are written in the other
language than the passage. Targets are drawn with a fixed seed
(`examples/draw_targets.rs`).

The metric that counts is **the expected note among the five returned**, because
five notes is what `engram search` shows. A proportion on *n* cases has a standard
error of √(p(1−p)/n): 14 points at 12 cases, 10 at 24, 8 at 36. Two variants that
differ by two cases out of twenty-four are not distinguished, and the tables say so.

## Retrieval quality

Same index for every column; ablations remove signals before ranking
(`ENGRAM_NO_QUESTIONS=1`, `ENGRAM_ID_BONUS=0`, `ENGRAM_LEXICAL=1`).

| family | words only | text only | + identifier bonus | + indexed questions (default) |
|---|---|---|---|---|
| topic of a note (24) | 33 % | 83 % | 83 % | 83 % |
| buried detail (24) | 54 % | 58 % | 58 % | **75 %** |
| named identifier (12) | 75 % | 75 % | **92 %** | **100 %** |
| first benchmark (36) | 19 % | 81 % | 81 % | 81 % |

Reading: each signal moves only the family it was designed for. The identifier bonus
adds seventeen points on identifiers and nothing elsewhere. Indexed questions add
seventeen points on buried details, lift the right passage from 46 % to 62 % on that
family, and leave the topic families unchanged. The words-only baseline is a
well-ranked grep: it nearly suffices on identifiers, which is why the engine keeps a
lexical signal, and it does not cross the language boundary.

Discarded after measurement, all within noise: vector centering (83 / 42 / 78 on the
first sixty queries), maximal marginal relevance reranking (50 / 58 / 81).

Compared engines on the sixty queries measured with all three (text only):

| engine | topic | detail | first benchmark |
|---|---|---|---|
| granite-embedding-278m, Q8 (= F32) | 75 % | 42 % | 81 % |
| static embeddings, 128 M parameters | 58 % | 25 % | 53 % |
| grep | | | 19 % |

## Memory

Resident memory of the warm process, one intervention at a time, concordance with
the reference and benchmark identical at each step:

| step | resident | change |
|---|---|---|
| first engine, F32 weights, one tokenizer | 1 842 MB | |
| embedding table read from the mapped file instead of loaded | 747 MB | −1 095 |
| Q8_0 on the linear layers, F32 activations | 512 MB | −235 |
| compact tokenizer (hash map and Viterbi) | 213 MB | −299 |
| 4 964 questions indexed, index ×4 | 243 MB | +30 |
| native SentencePiece model, no JSON parse | **198 MB** | −45 |

An isolated call (load, answer, exit) peaks at 310 MB and takes 0.21 s.

## Latency and throughput

| indicator | isolated call | warm process |
|---|---|---|
| latency per search | 0.21 s | 0.10 s (0.09 to 0.12 over ten runs) |
| CPU per search | 0.42 s | ≈ 0.40 s, client included |
| resident between calls | 0 | 198 MB |
| serial throughput | ≈ 4 req/s | 9.3 req/s |
| first call after an index write | 2.06 s | 0.24 s |

Model load 117 ms (28 ms for the vocabulary), query encoding 47 ms, index of 20.9 MB
read in about 15 ms, full re-embedding of 1 661 paragraphs 313 s on eight threads
(841 s on one). Ranking 6 625 vectors of 768 dimensions: 4 ms.

## Fidelity of Q8 to F32

One paragraph in six of the corpus (278), embedded in F32 then in Q8_0:

| tokens | n | min cosine | median |
|---|---|---|---|
| under 64 | 7 | 0.99990 | 0.99993 |
| 64 to 127 | 27 | 0.99988 | 0.99992 |
| 128 to 255 | 75 | 0.99988 | 0.99992 |
| 256 to 512 | 169 | 0.99988 | 0.99992 |

The gap does not grow with length: quantisation errors are zero-mean and cancel in
the dot product. Dynamic int8 (quantised activations) had cost eight to eleven points
of recall on the same benchmark; Q8_0 on weights only is a different object.

## Tokenizer

| implementation | load | resident | parity |
|---|---|---|---|
| reference crate (`tokenizers`) | 237 ms | 380 MB | reference |
| compact Unigram, from `tokenizer.json` | 52 ms | 79 MB peak, 30 MB resident | 0 mismatch on 2 027 texts |
| compact Unigram, native `sentencepiece.bpe.model` | 28 ms | 43 MB peak, 30 MB resident | 0 mismatch on the same texts |

## What was tried and removed

- **Metal, F16.** Five times faster encoding once warm, but 9.4 s of kernel
  compilation per process and thirteen all-NaN vectors on real paragraphs of 100 to
  350 tokens (layer-norm sum of squares overflows F16). Removed; the encoder now
  refuses any non-finite vector.
- **BF16 on CPU.** No matmul in the inference library.
- **Static embeddings.** 20 to 30 points of recall below the transformer.
- **The library ModernBERT graph.** Its cosine with the reference stayed at 0.85
  because of a hard-coded activation; replaced by a graph written after the reference
  implementation, at 1.000000 (see the last section).

## Reproduce

```sh
engram index                                   # build the index on your notes
cargo run --release --example draw_targets 24  # JSON skeleton of blind targets, fill the queries
cargo run --release --example bench            # the tables above, on your corpus
ENGRAM_NO_QUESTIONS=1 cargo run --release --example bench
ENGRAM_LEXICAL=1 cargo run --release --example bench
cargo run --release --example q8_fidelity 6    # Q8 versus F32 on one paragraph in six
cargo run --release --example load_probe       # load and encode times
cargo run --release --example rss_probe        # resident memory step by step
```

## The ModernBERT model

`ibm-granite/granite-embedding-small-english-r2` (ModernBERT, 12 layers, 384
dimensions, 8192-token window, English) on the same corpus, text only, no
questions: the long window merges paragraphs into 1 117 chunks instead of 1 661, so
the "right passage" column is not comparable.

| family | words only | multilingual 278M, text only | ModernBERT small English, text only |
|---|---|---|---|
| topic of a note (24) | 33 % | 83 % | 83 % |
| buried detail (24) | 54 % | 58 % | 54 % |
| named identifier (12) | 75 % | 75 % | **100 %** |
| first benchmark (36), one third in French | 19 % | 81 % | 72 % |

Reading: on an English-first corpus the smaller model holds the topic family and
wins on identifiers without any lexical bonus; it loses nine points on the family
where a third of the queries are in French, which is what an English model should
lose. Isolated search 0.43 s and 285 MB peak: the byte-level BPE vocabulary
(180 000 entries, 413 000 merges) is still parsed from JSON at every start, which
the multilingual model no longer pays.

Concordance with the reference vectors: cosine 1.000000 in F32, 0.99986 in Q8. The
library implementation of the graph stalled at 0.85 on this model because it
hard-codes a GELU activation where the configuration says SiLU; the graph shipped
here reads the activation from the configuration.
