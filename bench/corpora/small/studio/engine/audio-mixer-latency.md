---
name: audio-mixer-latency
description: Audio mixer on its own thread at 48 kHz with a 256-sample buffer, 21 ms latency on desktop, underruns fixed by thread priorities
type: project
status: active
verified: 2026-03-30
---

## Setup

`givre-audio` mixes on a dedicated thread, 48 kHz, 256-sample buffer (5.3 ms), 64 voices maximum, with a lock-free command queue from the game thread (play, stop, set parameter). Sound banks are loaded by the streamer under the audio budget of [[asset-streaming-budget]].

Latency from a game event to the speaker, measured with a loopback cable and a click track: 21 ms on the desktop reference, 34 ms on the low-end target (its audio stack adds a 12 ms buffer we cannot shrink). Both under the 40 ms target set by the audio designer for the sailing feedback sounds.

## The underrun problem

February 2026 playtest: crackling on the low-end target when entering a new chunk. Underruns counted by the mixer (`audio.underruns`) spiked at chunk loads. Cause: the streaming thread ran at the same priority as the mixer thread and a decompression burst starved it for one buffer.

Fix (BR-355): the mixer thread runs at the highest priority the platform allows for a non-realtime thread; the streaming worker at below-normal. Underruns per hour on the low-end target: 40 to 0 over the same test route. We also moved the bank decompression to the streaming worker so the mixer never touches compressed data.

## Rules

- Nothing on the mixer thread allocates or locks. The command queue is the only communication; parameters are atomics.
- A voice that has not been heard (volume under -60 dB after distance attenuation) for 2 seconds is virtualised: it keeps its position in the sound but stops mixing. 64 voices are enough with this.
- Sound designers tune in the editor with the same mixer code, not a separate tool, so what they hear is what ships. The editor's hot reload of banks is the tools team's work.
