# Local inference backends: Atomic Chat, TurboQuant, ternary and 1-bit models

Date: 2026-11-10. Status: research report, not an architecture decision.

Requirement this serves (operator, 2026-11-10): local inference built into the app, offline, on as many platforms as
possible, from a strong GPU to no GPU; Ollama replaced in-app by the llama.cpp that Atomic Chat uses.

## How to read this report

Every statement carries one of three labels.

| Label | Meaning |
|---|---|
| **Confirmed** | Extracted from the named source and passed a three-reader check (at least 2 of 3 agreed) |
| **Unchecked** | Extracted from the named source by one reader; the cross-check did not run (the research run ended on the usage limit) |
| **Inference** | My conclusion from the statements above; not stated by any source |

The research run fetched about 20 sources and extracted about 100 claims. Eleven were cross-checked, one was
rejected, the rest are Unchecked. No measurement in this report was made on this machine or on a phone.

## 1. What Atomic Chat is

| Statement | Label | Source |
|---|---|---|
| Apache-2.0 local AI app and inference engine; began as a fork of Jan (Menlo Research); desktop app built with Tauri | Confirmed | [1] |
| Ships for macOS 13.6+, Windows 10/11 x64, Linux x86_64 (glibc 2.35+), iOS and Android | Confirmed | [1] |
| The mobile apps are described as native, "not wrappers" | Confirmed | [2] |
| There is no PWA or web build described (a `web-app` folder exists in the repository, undescribed) | Confirmed | [1][2] |
| Twelve inference engines sit behind one OpenAI-compatible HTTP API at `http://localhost:1337/v1` | Confirmed | [1] |
| The default engine on Windows and Linux is upstream `ggml-org/llama.cpp` | Confirmed | [1] |
| Other engines: MLX-VLM, Apple Foundation Models, Foundry Local, LiteRT-LM, GenieX, FastFlowLM, TensorRT-LLM, vLLM, SGLang | Confirmed | [1] |
| "Atomic Chat's backend is its own TurboQuant llama.cpp fork" | **Rejected** (1 of 3) | [2] |
| Its engine table lists no engine for Android or iOS, and does not mention embeddings | Unchecked | [1] |
| Atomic Agent exposes a Tauri sidecar speaking newline-delimited JSON, plus the HTTP server | Unchecked | [2] |

**Inference.** "Atomic Chat's llama.cpp" is not one thing. On Windows and Linux it is stock upstream llama.cpp run as
a local server; TurboQuant and Bonsai are two further, separate llama.cpp builds. What runs on its Android app is not
established by any source read. There is no Atomic Chat PWA to copy.

## 2. TurboQuant (KV-cache compression)

| Statement | Label | Source |
|---|---|---|
| A KV-cache compression method from Google Research (ICLR 2026): PolarQuant quantises key/value vectors to 3 bits, QJL adds a 1-bit correction | Confirmed | [3] |
| It compresses only the KV cache, not model weights, so it applies to any GGUF model without re-conversion | Confirmed | [3] |
| In Atomic Chat it is delivered by a separate fork, `atomic-llama-cpp-turboquant`, with cache types `turbo3` / `turbo4` | Confirmed | [1] |
| Kernels exist for Metal, CUDA, Vulkan and HIP; CPU is also listed; no WebGPU or WASM kernel is mentioned | Confirmed | [1][2] |
| Available on macOS, Windows and Linux; no engine row lists Android or iOS | Confirmed | [1] |
| Vendor claims: up to about 4.3x smaller KV cache [1]; "peak RAM falls by 6x, output unchanged" [3] | Confirmed that the vendor claims it | [1][3] |
| Upstream llama.cpp PR #21089 (types `tbq3_0` / `tbq4_0`) was closed unmerged on 2026-06-02 | Confirmed | [4] |
| That PR was CPU-only | Confirmed | [4] |
| In that PR's own CPU benchmark, the TurboQuant types roughly halved generation speed against f16 / q4_0 (about 8.3 against 15.7 tokens/s) and `tbq4_0` saved little over `q4_0` (16.25 against 18.00 MiB) | Unchecked | [4] |
| Maintainers said no evidence showed it beats existing cache quantisation at similar bit width | Unchecked | [4] |
| The accuracy claim rests on Google's benchmarks, not on Atomic Chat's measurements of its own build | Unchecked | [3] |
| An independent evaluation attributed to the vLLM team found accuracy drops at 3 bits on reasoning and very long context | Unchecked | [5] |
| A review article states TurboQuant is desktop-only in Atomic Chat, not in the mobile builds | Unchecked | [6] |

**Inference.** TurboQuant is a desktop-only option that requires carrying a llama.cpp fork. It does not exist in
upstream llama.cpp, so it does not exist in wllama or in any build that tracks upstream. Its benefit is memory for
long contexts; the navigator's use (short embedding inputs, short chat context) gains little from it. Upstream
llama.cpp already offers `q8_0` and `q4_0` KV-cache types on every platform.

## 3. Ternary and 1-bit models

### Bonsai (PrismML)

| Statement | Label | Source |
|---|---|---|
| Atomic Chat runs Bonsai through a third, dedicated build (PrismML llama.cpp) because "the other llama.cpp builds don't load" its 1- and 2-bit tensor types; listed for macOS, Windows x64, Linux x64 only | Confirmed (2 of 3) | [1] |
| Ternary-Bonsai-1.7B: ternary weights, based on Qwen3-1.7B, 32,768-token context, Apache 2.0, GGUF | Unchecked | [7] |
| Its repository ships two files: `PQ2_0` (442 MB) needing the Prism fork, and `Q2_0` g64 (467 MB) that runs on upstream llama.cpp | Unchecked | [7] |
| Vendor speed for 1.7B on an Apple M4 Pro: 229 tokens/s on Metal, 123 tokens/s on CPU with 10 threads | Unchecked | [7] |
| Vendor benchmark average for 1.7B: 58.47 against 66.57 for full-precision Qwen3 1.7B | Unchecked | [7] |
| Ternary Bonsai sizes 8B / 4B / 1.7B; 8B is 1.75 GB; 27 tokens/s on an iPhone 17 Pro Max | Unchecked | [8] |
| Bonsai 27B: ternary about 5.9 GB theoretical (7.2 GB on disk), 1-bit about 3.9 GB, about 11 tokens/s on an iPhone 17 Pro Max | Unchecked | [9] |
| The llama.cpp WebGPU backend (LlamaWeb, delivered through wllama) supports the `q1_0` format introduced for 1-bit Bonsai | Unchecked | [10] |

**Open contradiction.** The Atomic Chat README says upstream llama.cpp cannot load Bonsai; the model card says one of
the two files does. Both can be true (different files), but this was not cross-checked and decides whether Bonsai can
be part of a single-backend plan. It needs one direct test: load the `Q2_0` g64 file in the vendored wllama.

### BitNet (Microsoft) and llama.cpp's own ternary types

| Statement | Label | Source |
|---|---|---|
| `bitnet.cpp` is a llama.cpp derivative with its own quantisation types (`i2_s`, `tl1`, `tl2`); a separate build, CPU-focused; no Android, WASM or mobile target in its README | Unchecked | [11] |
| Reported CPU speedups against a full-precision baseline: 2.37-6.17x on x86, 1.37-5.07x on ARM | Unchecked | [11] |
| A critical review notes those speedups are against FP16, not against 4-bit, and that the released BitNet 2B4T file is about 1.19 GB, not 0.4 GB | Unchecked | [12] |
| Upstream llama.cpp has had ternary types `TQ1_0` (1.69 bits) and `TQ2_0` (2.06 bits) since September 2024, with AVX2 and ARM NEON CPU kernels | Unchecked | [13] |
| On a 2017 Android Cortex-A53, `TQ2_0` ran at about 1.8x the throughput of `Q4_K` in the kernel benchmark; no GPU or WASM path was benchmarked | Unchecked | [13] |
| The BitNet paper says `TQ1_0` / `TQ2_0` are not lossless for BitNet b1.58 specifically | Unchecked | [14] |

**Inference.** Three different "1-bit" ecosystems exist and they do not share a runtime: Bonsai (PrismML fork, with a
possible upstream-compatible file), BitNet (bitnet.cpp), and llama.cpp's own `TQ` types. Only the last is certain to
run on a stock llama.cpp everywhere.

## 4. The web-view path already in this repository

| Statement | Label | Source |
|---|---|---|
| wllama runs llama.cpp in the browser on CPU through WebAssembly SIMD; supports embeddings | Unchecked | [15] |
| A single GGUF file cannot exceed 2 GB; the README recommends splitting into 512 MB parts | Unchecked | [15] |
| Multi-threading needs the page to send cross-origin isolation headers (COOP and COEP); otherwise it runs single-threaded | Unchecked | [15] |
| From wllama 3.1, WebGPU is used automatically with all layers offloaded | Unchecked | [15] |
| Against WebLLM and Transformers.js, the llama.cpp WebGPU path used 29-33% less memory and decoded 45-69% faster, but processed prompts 21-51% slower | Unchecked | [10] |
| Chromium ships WebGPU by default on Windows x64 since version 113 | Unchecked | [16] |
| On Linux, Chromium enables WebGPU by default only on recent Intel and NVIDIA-on-Wayland; Tauri on Linux uses WebKitGTK, which the source does not cover | Unchecked | [16] |
| On Android, Chrome ships WebGPU for ARM / Qualcomm / Intel GPUs on Android 12+; Android WebView is not covered by the source | Unchecked | [16] |
| No Tauri plugin embedding llama.cpp on Android was found; the nearest analogue is a Capacitor plugin using the NDK and CMake | Unchecked | [17] |

This repository vendors wllama at `VENDOR/wllama3` and has the crate `crates/inference-wllama`.

## 5. Matrix

All cells are **Inference** from sections 1-4. "Native llama.cpp" means upstream llama.cpp compiled into or beside
the app. "wllama" means the same llama.cpp compiled to WebAssembly, running in the app's web view or a browser.

### 5.1 What runs where

| Platform | Strong discrete GPU | Integrated or mobile GPU | CPU only |
|---|---|---|---|
| Linux desktop (Tauri) | Native llama.cpp with CUDA or Vulkan. TurboQuant fork possible | Native llama.cpp with Vulkan | Native llama.cpp (AVX2). wllama also works, slower |
| Windows desktop (Tauri) | Native llama.cpp with CUDA or Vulkan. TurboQuant fork possible. wllama with WebGPU also plausible (WebView2 is Chromium) | Native llama.cpp with Vulkan, or wllama with WebGPU | Native llama.cpp (AVX2). wllama also works, slower |
| Android (Tauri) | not applicable | Native llama.cpp through the NDK, CPU first; Vulkan and WebGPU-in-WebView are unproven here | Native llama.cpp through the NDK (NEON), or wllama in the web view |
| PWA / browser | wllama with WebGPU where the browser ships it | wllama with WebGPU where shipped, otherwise CPU | wllama on WebAssembly; single-threaded unless the host sends isolation headers |

### 5.2 Which features survive on which backend

| Backend | Desktop | Android | PWA | Embeddings | Ordinary GGUF (Q4-Q8) | llama.cpp ternary (`TQ1_0` / `TQ2_0`) | Bonsai | BitNet b1.58 official | TurboQuant KV cache |
|---|---|---|---|---|---|---|---|---|---|
| Upstream llama.cpp, native | yes | yes (NDK build needed) | no | yes | yes | yes, CPU kernels | `Q2_0` g64 file only, unchecked | lossy per [14] | no |
| wllama (upstream llama.cpp in WASM) | yes | yes | yes | yes | yes, files up to 2 GB or split | yes in principle, speed unmeasured | `q1_0` on WebGPU per [10], unchecked | lossy per [14] | no |
| Atomic TurboQuant fork | yes | not listed | no | not stated | yes | not stated | no | not stated | yes |
| PrismML llama.cpp fork | yes | not listed | no | not stated | presumably | not stated | yes, all files | not stated | has its own 4-bit KV quantiser per [9] |
| bitnet.cpp | yes, CPU | not listed | no | x86 only per [11] | no | no | no | yes | no |

### 5.3 Combinations

| Option | What it is | Covers | Cost |
|---|---|---|---|
| **A. One engine, two packagings** | Upstream llama.cpp only: native on desktop and Android, wllama for the PWA. One set of GGUF model files for every platform | Every platform and every hardware level in 5.1 | One Android NDK build of llama.cpp to create and maintain. No TurboQuant. Bonsai limited to the upstream-loadable file, if the test passes |
| **B. wllama everywhere** | The vendored wllama in the web view on all platforms, no native engine | Every platform with the least new build work; matches what `crates/inference-wllama` was written for | Slowest on desktop and Android CPUs; 2 GB file limit; GPU use depends on the web view (not available in WebKitGTK on Linux as far as the sources show); threading depends on isolation headers |
| **C. A plus desktop extras** | Option A, with the TurboQuant fork and/or the PrismML fork offered as additional desktop engines, the way Atomic Chat does | Adds long-context memory savings and all Bonsai files on desktop with a GPU | Two or three llama.cpp builds to keep in step on desktop; features that exist on desktop and not on Android or PWA |
| **D. Copy Atomic Chat's shape** | A local HTTP server per engine behind one OpenAI-style API | Desktop, as Atomic Chat proves | Its mobile engine is unknown; a localhost server is a poor fit for Android and impossible in a PWA |

**Best across all platforms: A.** It is the only option where the same engine and the same model files run in every
cell, including no-GPU machines, and it is what Atomic Chat itself defaults to on Windows and Linux. The navigator's
current HTTP calls to Ollama map onto it directly, because llama.cpp's server speaks the same OpenAI-style embedding
and chat endpoints the navigator already has a code path for.

**Individually better:**

- Desktop with a strong GPU and long contexts: C (TurboQuant fork), accepting the extra build.
- Smallest capable chat model on desktop: C (PrismML fork) for the full Bonsai range.
- PWA and any device where a native build is not available: B, which is A's PWA half.
- Fastest route to something working inside the existing app: B, since wllama is already vendored.

## 6. Models for the navigator (Inference)

The navigator needs an embedding model always, and a chat model for its agent panel.

| Need | Candidate | Why | Caveat |
|---|---|---|---|
| Embeddings | A small GGUF embedding model in the 0.3-0.6B range, such as `qwen3-embedding:0.6b`, which this machine's Ollama already serves at 1024 dimensions | Runs on upstream llama.cpp and wllama alike; small enough for a phone | Speed on a phone CPU and in WebAssembly not measured |
| Chat, smallest | Ternary-Bonsai-1.7B, `Q2_0` g64, 467 MB | Under wllama's 2 GB limit without splitting; vendor reports usable quality | Upstream loading is Unchecked; quality figures are the vendor's |
| Chat, safe | Any 1-2B conventional model at Q4 | Certain to load on every backend in 5.2 | Roughly twice the size of the ternary file |

The 1-bit embedding models in bitnet.cpp are x86-only per [11] and need bitnet.cpp, so they do not fit option A.

## 7. What is not known

1. Which engine Atomic Chat's Android app uses.
2. Whether the Bonsai `Q2_0` g64 file loads in upstream llama.cpp and in the vendored wllama.
3. Whether WebGPU is available inside the Tauri web view on Android and on Linux (WebKitGTK).
4. Any speed figure on Android or in WebAssembly for ternary models; all vendor numbers are Apple hardware.
5. Which upstream llama.cpp revision `VENDOR/wllama3` contains, and therefore which quantisation types it has.

Items 2, 3 and 5 can each be settled by one direct test on this machine or a phone.

## Sources

1. https://github.com/AtomicBot-ai/Atomic-Chat
2. https://github.com/AtomicBot-ai
3. https://atomic.chat/turboquant
4. https://github.com/ggml-org/llama.cpp/pull/21089
5. https://dev.to/carryologist/turboquant-four-months-later-chasing-googles-6x-vram-claim-into-the-wild-4gh1
6. https://webscraft.org/blog/atomic-chat-scho-tse-ta-yak-pratsyuye
7. https://huggingface.co/prism-ml/Ternary-Bonsai-1.7B-gguf
8. https://prismml.com/news/ternary-bonsai
9. https://neurohive.io/en/state-of-the-art/h1-bonsai-27b-1-bit-weights-put-a-27b-parameter-model-on-a-smartphone-for-the-first-time-h1/
10. https://arxiv.org/html/2605.20706
11. https://github.com/microsoft/BitNet
12. https://wavect.io/blog/bitnet-cpu-llm-ram-requirements/
13. https://app.semanticdiff.com/gh/ggml-org/llama.cpp/pull/8151/overview
14. https://arxiv.org/pdf/2502.11880
15. https://github.com/ngxson/wllama
16. https://github.com/gpuweb/gpuweb/wiki/Implementation-Status
17. https://github.com/arusatech/llama-cpp

Source-to-claim attribution for Unchecked rows was reconstructed from the research run's search and extraction
records; where a claim could belong to two similar pages (for example two Atomic Chat pages), the more specific page
is cited.
