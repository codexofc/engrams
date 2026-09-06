---
name: gpu-vs-cpu-inference-cost
description: The two GPU nodes (2 300 EUR a month amortised) are justified by OCR alone at 4 minutes per document on CPU versus 6 seconds on GPU, ML inference stays on CPU because gradient boosted models cost 0.4 ms per prediction there, the 2026 numbers and the decision not to buy a third GPU node for the ML team
type: project
status: active
verified: 2026-05-20
---

# GPU or CPU: what runs where and what it costs

## The question

In March 2026 the ML team asked for a GPU node "for experiments and maybe serving". The OCR pipeline already had two GPU nodes (`hf-ocr-01` and `-02`, one mid-range datacentre GPU with 16 GB each, bought in 2025). A third node is about 1 150 EUR a month amortised plus power, the largest single item anyone had asked for since the storage drawers. HF-4730 was the analysis; this note is what it found.

## OCR: the case for the GPUs

The OCR pipeline (layout detection plus text recognition on PODs and CMRs) processes about 9 000 documents a day, 40 000 on the last day of the month when carriers upload everything at once.

### Measured on the same model, same documents

| | CPU (8 cores reserved on a worker node) | GPU node |
|---|---|---|
| time per document, p50 | 4 min 10 s | 6.2 s |
| documents per hour per unit | 14 | 580 |
| units needed for 9 000 a day within 4 h | 160 cores | 1 GPU node at 65 % |
| units needed for 40 000 in a day | 700 cores | 2 GPU nodes at 90 % |
| monthly cost of the units | 160 cores × 21 EUR = 3 360 (normal day) to 14 700 (peak sized) | 2 × 1 150 = 2 300 |

The CPU column is not even a serious option: sizing for month-end on CPU would be 22 nodes' worth of requests for one workload. The GPUs pay for themselves on OCR alone, and month-end is why there are two rather than one. Between month-ends the second node is at 10 % and the ML team's experiments run on it under a lower-priority class, which is the answer to half of their request.

## ML inference: the case against

`ml-infer` serves ETA, price suggestion and demand forecast predictions from gradient boosted tree models, about 300 predictions a second at peak, p50 latency 4 ms of which 0.4 ms is the model.

| | CPU (current, 4 pods × 500 m) | GPU (tested, one pod on `hf-ocr-02`) |
|---|---|---|
| model time per prediction | 0.4 ms | 0.9 ms (batching overhead, small trees do not benefit) |
| throughput per unit | 2 500 / s per core | 1 100 / s per GPU for single predictions, 40 000 / s in batches of 512 |
| monthly cost for the peak | 2 cores × 21 EUR = 42 EUR | 1 150 EUR |

Batching would make the GPU fast, and batching adds latency (waiting for 512 requests at 300 a second is 1.7 s) that the ETA consumer cannot accept. For tree models at this volume, CPU is 25 times cheaper and faster. This matches the ML team's own preference for tree models and their 2025 finding that a neural ETA was one minute better at twenty times the serving cost.

## Training

The weekly retrains (ETA, price, demand) take 20 to 40 minutes each on 16 CPU cores of the pool, about 3 core-days a week, 30 EUR a month in the allocation. On a GPU they would take 5 minutes and change nothing: a retrain that finishes at 03:20 instead of 03:55 is not worth a node. The experiments that would benefit (the occasional neural baseline the team runs to confirm it still loses) run on `hf-ocr-02` between month-ends.

## Decision

No third GPU node. The ML team has:

- a `PriorityClass` `ml-experiments` on the GPU nodes, preempted by OCR, with a monthly report of the hours used (April: 190 GPU-hours, all on `-02`, 0 preemptions during month-end because the team knows the calendar);

- CPU inference as before, with the request sizing from the [[rightsizing-2025-q4-requests-limits]] pass (4 pods × 500 m is generous already);

- a written trigger for revisiting: if a model family that needs a GPU to serve beats the tree baseline by a margin the consumer cares about, on a backtest, the node is bought. That is the ML team's own promotion rule stated in cost terms.

## Where it shows in the tables

The `ml` and `ocr` lines of [[per-service-cost-table-q2-2026]] are 8 600 EUR a month together, of which 2 300 is the two GPU nodes and 4 100 is the feature store and `ml-infer` compute and storage. The share of GPU time used by ML experiments (about 12 % of `-02`) is noted, not reallocated ([[cost-allocation-labels]]): it is capacity that exists for month-end and would be idle otherwise.

## What would change the answer

- OCR volume doubling: a third GPU node for month-end, unrelated to ML.

- A document type that needs a larger model (handwritten CMR fields have been discussed): possibly a node with more memory, again for OCR.

- Batch scoring workloads (scoring all 3 M historical loads with a new model for a backtest) taking more than a day on CPU: the ML team does this twice a year and it takes 6 hours on 16 cores; not yet.
