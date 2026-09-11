# Tuning the embedding model on a team's own memory

An experiment plan, not a feature. The engine does not train anything, as the
roadmap states: training happens outside, once, and produces weights that the model
registry loads like any other entry. This document says what the training data is,
what gain to expect, what it costs, how the result is judged, and where it stops.

## Why it is worth an experiment

On my private corpus (96 blind queries in four families) the default model reaches
83 / 58 / 75 / 81 % top 5 on topic, detail, identifier and the first bench. The weak
family is the buried detail at 58 %. The largest model of the registry, e5-large,
five times bigger and 0.55 s per isolated call, brings the detail family to 71 %.
That gap of 13 points is the visible ceiling of what a better encoder buys on this
corpus. A 278 M model tuned on the team's own pairs can reasonably recover half to
all of it, at no inference cost, which is the whole point: the small model stays
small.

What the literature measures on equivalent recipes, with the sources in the
benchmark page:

| study | model | data | measured gain | cost |
|---|---|---|---|---|
| Schmid, financial filings | bge-base, 109 M | 7 000 pairs, 4 epochs | nDCG@10 0.768 to 0.825, and the tuned 128-dim model beats the 768-dim baseline | 3 min 26 s on a rented GPU, 0.07 $ |
| NVIDIA, internal docs | 1 B | LLM-generated queries | recall@10 0.630 to 0.693, Jira recall@60 0.751 to 0.951 | 2 to 3 h for 500 documents, one GPU |
| Databricks, three enterprise sets | gte-large, 434 M | enterprise Q&A | recall@10 +4 % to +88 % by domain, no gain where the base model already knew the domain, beats reranking on two sets out of three | not given |
| CustomIR, known corpora | small models | synthetic pairs, LLM-verified hard negatives | up to +2.3 points recall@10 | not given |
| cross-encoder to bi-encoder distillation | small student | teacher scores, Margin-MSE | recall@3 +19.7 % on SQuAD, +3.6 % on a RAG set | not given |

The spread is wide and depends on how far the domain sits from the model's training
data. A team's engineering memory, with its identifiers, its product names and its
two languages inside one note, is far from it, which is the case where the studies
report the larger gains.

## The training data already exists

- **4 677 generated questions** on 1 559 paragraphs of my memory, each one a
  (question, paragraph) pair, produced by the questions pipeline that runs in every
  installation. A team of thirty produces two orders of magnitude more without anyone
  writing a line.
- **The learned table**: explicit confirmations (`kept learn`) and reads that
  followed a search that missed, which are human-labelled positives.
- **Blind evaluation sets** kept out of training: the 96 queries of the private
  bench, and the 60 + 30 queries of the two public corpora, written from the target
  passages by someone who did not write the notes.
- **Hard negatives** come for free from the engine: for each question, the top 5
  notes that are not the target, verified by the same LLM that wrote the question
  when the pair is ambiguous, as CustomIR does.

## Two recipes

1. **Fine-tuning on the team's pairs.** Contrastive loss with in-batch negatives
   (MultipleNegativesRankingLoss) plus the Matryoshka loss so that 256 and 128
   dimensions stay usable. 1 to 2 epochs, as NVIDIA recommends against overfitting,
   batch of 64 with the cached variant of the loss. The cheapest recipe and the data
   exists.
2. **Distillation from a teacher.** e5-large, already in the registry, or a
   cross-encoder, scores (query, paragraph) pairs and the small model learns the
   score margins (Margin-MSE). Costs one pass of the teacher over the corpus first:
   6 658 chunks at 0.55 s is about one hour of CPU. Worth it if recipe 1 stalls on the
   detail family, since the teacher sees both texts at once.

## Costs, measured or estimated

| item | figure | status |
|---|---|---|
| training, recipe 1, 5 000 pairs, 278 M model | minutes on a rented GPU, about ten minutes on Apple Silicon | estimate from Schmid's 3 min 26 s on 109 M |
| teacher pass, recipe 2 | about 1 h CPU on 6 658 chunks | from the measured 0.55 s per call |
| index rebuild after a new model | 722 s for 3 424 chunks, about 25 min on my corpus | measured |
| in the server mode | re-embedding in the background, no downtime | by design, see SERVER-MODE.md |
| tooling | a training script with sentence-transformers, outside the engine | decision below |

On tooling: candle can do backpropagation, but no fine-tuning recipe for XLM-RoBERTa
exists in Rust and writing one is days of work for no product value. The training
script lives in `training/` as a documented, reproducible, one-off procedure that
takes the pairs the engine exports (`kept export --pairs`) and writes safetensors.
The engine never runs it. The only thing the product needs is what it has: a model
registry entry with the weights hash.

## Protocol

1. Export the pairs of the training corpus. Remove every pair whose paragraph is the
   target of a blind query. The blind sets are never seen in training.
2. Train recipe 1. Keep the weights at the epoch with the best recall on a held-out
   10 % of the generated questions, never on the blind sets.
3. Register the weights as `granite-multilingual-team` with its hash, rebuild the
   index, replay `examples/bench` on the private corpus and `scripts/bench-corpus.sh`
   on both public corpora, text only and with questions, and the concordance test.
4. Measure both languages separately: a third of the blind queries are in the other
   language, and a drift of the multilingual ability is the classic failure of a
   fine-tuning on one team's text.
5. If the detail family gains fewer than 5 points, run recipe 2 with e5-large as the
   teacher and repeat step 3.

## Acceptance

- At least +5 points top 5 on the detail family of the private corpus, and no family
  losing more than 2 points, in either language.
- No loss on the public corpora, which were not in training: they are the guard
  against a model that only knows my memory.
- The Q8 quantised version of the tuned model within 0.999 cosine of its F32 version,
  as the current model is.
- Isolated call time unchanged: the model is the same size.

If any line fails, the tuned model is not registered as a default anywhere and the
result goes in BENCHMARKS.md as a negative result with its numbers.

## Where it stops

- A model trained on a team's memory is that team's artefact. The public product
  ships the base model. A team that wants its own runs the procedure on its server,
  and the server picks up the registry entry.
- Training is a change of weights, not of the engine. Nothing in the retrieval
  pipeline, the learning tables or the file formats depends on which weights run.
- This experiment comes after the server mode, not before: on a single person's
  memory the gain is a few points on one family, on a team's memory it is a single
  model, trained once, that improves every developer's results at once.
