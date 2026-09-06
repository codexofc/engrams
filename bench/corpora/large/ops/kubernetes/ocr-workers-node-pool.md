---
name: ocr-workers-node-pool
description: Document OCR (CMR and POD text extraction) runs on two GPU nodes hf-ocr-01 and -02 as a Deployment of ocr-worker pods consuming the hf.ocr queue, 1 GPU per pod via the device plugin, throughput 40 pages/min per pod, with the kernel module pinning that bit us
type: project
status: active
verified: 2026-05-30
---

# OCR worker node pool

The data platform's OCR service extracts text from uploaded CMRs and PODs (typed fields for the accounting integration, and a searchable text for support). It is the only GPU workload on `hf-main`.

## Hardware and scheduling

Two nodes, `hf-ocr-01` (rack A) and `hf-ocr-02` (rack B), 16 cores, 64 GB, one mid-range datacentre GPU each (16 GB VRAM). Tainted `workload=ocr:NoSchedule`, labelled `hf.example/pool: ocr`, `nvidia.com/gpu.present: "true"`. See [[node-pools-and-taints]].

The device plugin (DaemonSet in `kube-system`, one of the documented privileged exemptions in [[rke2-cis-hardening]]) advertises `nvidia.com/gpu: 1` per node. The `ocr-worker` Deployment in namespace `data` requests `nvidia.com/gpu: 1`, so at most one pod per node, two in total. No time-slicing: the model is loaded once per pod and uses 11 GB of VRAM.

## Workload

- `ocr-worker` consumes RabbitMQ queue `hf.ocr` (a message per document, with the object key). It downloads the file from the object store, runs layout detection and text recognition, posts the result to the API (`POST /internal/documents/{id}/ocr`) and acks.

- Throughput measured: 40 pages per minute per pod on typical scanned CMRs (A4, 200 to 300 dpi). Daily volume is 6 000 to 9 000 pages, so the two pods are busy about 2 hours a day, mostly between 17:00 and 20:00 when drivers upload.

- If both nodes are down the queue accumulates; the API does not wait for OCR, documents are `AVAILABLE` without text and the text arrives later. A queue depth above 5 000 for an hour is a `warn`.

## The kernel module story

The GPU driver is a kernel module built out of tree. In 2025 the two OCR nodes had been provisioned from a different distribution channel than the general workers, and their kernel updated independently. During [[rke2-upgrade-1-31-to-1-32]] the RKE2 package upgrade pulled a kernel update on those nodes only, the module did not match the new kernel, and the OCR pods failed with `CUDA driver version is insufficient`. 40 minutes to rebuild with the vendor's DKMS package.

Since then:

- All nodes on the same channel, kernel updates only through the provisioning playbook, which runs `dkms status` after any kernel change and fails the play if the module is not built for the running kernel.

- The pre-drain check in [[runbook-node-drain-replace]] for OCR nodes: `nvidia-smi` must work after reboot before uncordon.

- The CIS `protect-kernel-defaults` setting required aligning the `sysctl` of these nodes with the others; the GPU driver's installer had changed `vm.overcommit_memory`, which kubelet then refused. Set back in the playbook.

## Model updates

The OCR model files (about 2 GB) are baked into the `ocr-worker` image, not mounted from a volume, so a rollout is an image change like any other component and the registry mirror caches it (see [[registry-harbor-mirror]]). A new model version is tested on a sample of 500 documents from staging against the previous version's output before promotion; the comparison script reports field-level agreement and it must not drop below 97 %.

## What we would do with a third GPU

Nothing yet. The two pods keep up. A third node was quoted and the answer was to wait until the daily busy time exceeds 4 hours, which is projected for 2027 at current growth.
