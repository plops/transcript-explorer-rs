# Transcript Explorer UI Tour

This document provides a walkthrough of the Transcript Explorer TUI, showcasing the main interface and its key features.

## 1. Initial Screen (Main List View)

When you first launch the application, you are presented with a paginated list of transcript summaries. The interface includes a header showing the total count, a search bar, the main list, and a preview pane for the selected entry.

```text
 Transcript Explorer   [12279 entries in 12096 groups]                                                      
                                                                                                            
────────────────────────────────────────────────────────────────────────────────────────────────────────────
┌ Search ──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🔍  Filter (/):                                                                                           │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transcripts ─────────────────────────────────────────────────────────────────────────────────────────────┐
│▸     1 ● **Lecture 11: Sparsity - Exploring Techniques for Efficient Deep Le…  $0.000                    │
│      2 ● **GPU-Accelerated Parallel Scan Algorithm: Optimizations and Analys…  $0.000                    │
│      3 ● **Lecture 21: Scan Algorithm Part 2 - Exploring Brent-Kung Scan and…  $0.000                    │
│      4 ● **Lecture 24: Scan at the Speed of Light - Optimizing Parallel Pref…  $0.000                    │
│      5 ● **The Real Reasons Behind Stefan Raab's Comeback**  $0.000                                      │
│      6 ● **Cheap and Convenient Computer Vision on a Budget with RV1106/RV11…  $0.000                    │
│      7 ● ## Rockchip NPU on a Budget: A Look at the Lockfox Max Pro Board fo…  $0.000                    │
│      8 ● Error: value error  $0.000                                                                      │
│      9 ● **Meta Unveils Orion: Augmented Reality Glasses Prototype Pushing t…  $0.000                    │
│     10 ● **Unveiling the Unrecognized Invasion: Russia's Quiet Encroachment …  $0.000                    │
│     11 ● **Octopus cyanea's Unexpected Social Hunting Strategy: Collaboratio…  $0.000                    │
│     12 ● **Gaussian Splats in Robotics: Enhancing Simulation, Navigation, an…  $0.000                    │
│     13 ● ## Recreating Genesis: From Java to C++ - A Full Game Remake in Ray…  $0.000                    │
│     14 ● **Remaking Genesis: A 12-Year Journey from Java to C++**  $0.000                                │
│     15 ● ## Remaking My First Game in C++ (Part 1)  $0.000                                               │
│     16 ● **TWiV 1150: Clinical Update with Dr. Daniel Griffin - Key Takeaway…  $0.000                    │
│     17 ● **Practical Memory Pool Based Allocators For Modern C++**  $0.000                               │
└───────────────────────────────────────────────────────────────────────────────────── Group 1-27 of 12096 ┘
┌ Selected Result Preview ─────────────────────────────────────────────────────────────────────────────────┐
│ID: 1  Host: 193.8.40.111  Model: gemini-1.5-pro-exp-0827  Cost: $0.000                                   │
│Tokens: In:25851 Out:827  Finished: 2024-09-25T15:10:44.768999 (duration: 20s)                            │
│Link:                                                                                                     │
│Summary:                                                                                                  │
│Lecture 11: Sparsity - Exploring Techniques for Efficient Deep Learning Inference and Training            │
│                                                                                                          │
│- 0:00 Introduction: The lecture focuses on sparsity in deep learning, aiming to reduce computational     │
│costs and model size by leveraging the fact that many weights in neural networks are unimportant.         │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
 ↑↓/PgUpDn Nav Space Expand f Filters /Search Enter Detail s Similar ? Help q Exit  12279 transcripts loaded
```

## 2. Search Results

Pressing `/` enters search mode. As you type, the list updates in real-time. Below is the view after searching for "GPU" and pressing `Enter`. Notice the result count dropped to 588 and entry #2 is now the focus.

```text
 Transcript Explorer   [588 entries in 588 groups]                                                          
                                                                                                            
────────────────────────────────────────────────────────────────────────────────────────────────────────────
┌ Search ──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🔍  Filter (/): GPU                                                                                       │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transcripts ─────────────────────────────────────────────────────────────────────────────────────────────┐
│      1 ● **Lecture 11: Sparsity - Exploring Techniques for Efficient Deep Le…  $0.000                    │
│▸     2 ● **GPU-Accelerated Parallel Scan Algorithm: Optimizations and Analys…  $0.000                    │
│      3 ● **Lecture 21: Scan Algorithm Part 2 - Exploring Brent-Kung Scan and…  $0.000                    │
│      4 ● **Lecture 24: Scan at the Speed of Light - Optimizing Parallel Pref…  $0.000                    │
│     15 ● ## Remaking My First Game in C++ (Part 1)  $0.000                                               │
│     56 ● **Circuits for AI/ML: Fundamentals and Design Considerations**  $0.000                          │
│     82 ● ## Meta's MovieGen: A New Era of Video and Audio Generation  $0.000                             │
│    123 ● ## Computer Architecture Lecture 11 Summary: Memory Controllers, Se…  $0.000                    │
│    125 ● **Lecture 33: Bitblast - Enabling Efficient Low Precision Computing…  $0.000                    │
│    126 ● **Lecture 33: Bitblas - Enabling Efficient Low Precision Computing …  $0.000                    │
│    159 ● ## Building a Video Shuffle Studio with AI: A Python Project Using …  $0.000                    │
│    248 ● ## Repurposing Old iMacs as Monitors: Exploring Solutions Beyond Ta…  $0.000                    │
│    269 ● **C++26: An Overview of Key Features and Changes**  $0.000                                      │
│    284 ● **Large Language Models: A Brief Explanation**  $0.000                                          │
│    302 ● **The Problem With Procedural Generation: Taming Chaos for Playabil…  $0.031                    │
│    330 ● **Video Generation: Exploring the Latest Advancements and Future Im…  $0.053                    │
│    350 ● Okay, here is a summary of the provided transcript in a bullet list…  $-0.043                   │
└─────────────────────────────────────────────────────────────────────────────────────── Group 1-27 of 588 ┘
┌ Selected Result Preview ─────────────────────────────────────────────────────────────────────────────────┐
│ID: 2  Host: 193.8.40.111  Model: gemini-1.5-pro-exp-0827  Cost: $0.000                                   │
│Tokens: In:28121 Out:604  Finished: 2024-09-25T12:12:12.044692 (duration: 15s)                            │
│Link:                                                                                                     │
│Summary:                                                                                                  │
│GPU-Accelerated Parallel Scan Algorithm: Optimizations and Analysis                                       │
│                                                                                                          │
│- 0:13 Introduction to Prefix Sum (Scan):  Scan operations take an input array and an associative operator│
│(e.g., sum, product, min/max) and produce an output array where each element is the result of applying the│
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## 3. Global Filter Widget

Pressing `f` opens the Global Filter Configuration. This view shows metadata statistics across the entire dataset (mean, median, etc., for cost and tokens) and allows adding complex filters based on metadata fields like `cost`, `model`, or `tokens`.

```text
 Global Filter Configuration  [12279 items total]                                                           
                                                                                                            
────────────────────────────────────────────────────────────────────────────────────────────────────────────
┌ Metadata Statistics ────────────────────┐┌ Active Global Filters (Applied to all views) ─────────────────┐
│ COST  (n=12279)                         ││No active global filters. All rows shown.                      │
│   Mean:   0.0291     StdDev: 0.0749     ││                                                               │
│   Min:    -0.2492    Max:    0.5000     ││                                                               │
│   Median: 0.0102     MAD:    0.0089     ││                                                               │
│   P5:     0.0000     P95:    0.0779     ││                                                               │
│                                         ││                                                               │
│ INPUT_TOKENS  (n=12279)                 ││                                                               │
│   Mean:   23970.4085 StdDev: 20493.2861 ││                                                               │
│   Min:    0.0000     Max:    369422.0000││                                                               │
│   Median: 18856.0000 MAD:    5742.0000  ││                                                               │
│   P5:     0.0000     P95:    57960.0000 ││                                                               │
│                                         ││                                                               │
│ OUTPUT_TOKENS  (n=12279)                ││                                                               │
│   Mean:   968.3086   StdDev: 655.9728   ││                                                               │
│   Min:    0.0000     Max:    18064.0000 ││                                                               │
│   Median: 893.0000   MAD:    216.0000   ││                                                               │
│   P5:     0.0000     P95:    1841.0000  ││                                                               │
│                                         ││                                                               │
│ MODELS                                  ││                                                               │
│   •                                     ││                                                               │
│   • gemini-1.5-flash-002                ││                                                               │
│   • gemini-1.5-flash-8b-exp-0924        ││                                                               │
│   • gemini-1.5-flash-exp-0827           ││                                                               │
│   • gemini-1.5-flash-latest             ││                                                               │
│   • gemini-1.5-pro                      ││                                                               │
│   • gemini-1.5-pro-002                  ││                                                               │
│   • gemini-1.5-pro-exp-02-05            ││                                                               │
│   • gemini-1.5-pro-exp-0801             ││                                                               │
│   • gemini-1.5-pro-exp-0827             ││                                                               │
│   ... and 40 more                       ││                                                               │
└─────────────────────────────────────────┘└───────────────────────────────────────────────────────────────┘
 a  Add Filter   d  Clear All   Esc  Back
```

## 4. Vector Similarity Search

Pressing `s` on a selected entry initiates a vector similarity search. The application finds the top 20 most similar transcripts based on their embeddings. The results are displayed with their similarity score (e.g., 0.882) and a preview pane for the currently selected similar entry.

```text
┌ Vector Similarity Search ────────────────────────────────────────────────────────────────────────────────┐
│ Similar to ID 1                                                                                          │
│ **Lecture 11: Sparsity - Exploring Techniques for Efficient Deep Learning Inferenc…                      │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Results (sorted by similarity) ──────────────────────────────────────────────────────────────────────────┐
│▸   1. 0.882  4653 This video features Ali Hassani discussing "neighborhood attention,…                   │
│    2. 0.878  4654 Here is the abstract and summary for the provided transcript.                          │
│    3. 0.875  7806 This video features a conversation with Nir Shavit, a professor at …                   │
│    4. 0.871   126 **Lecture 33: Bitblas - Enabling Efficient Low Precision Computing …                   │
│    5. 0.863  3045 This video features Perry Jang from UCSD's How AI Lab discussing "F…                   │
│    6. 0.860  3601 Here's the abstract and summary for the provided transcript:                           │
│    7. 0.859   125 **Lecture 33: Bitblast - Enabling Efficient Low Precision Computing…                   │
│    8. 0.858 11479 This presentation outlines the evolution of Google’s TPU SparseCore…                   │
│    9. 0.858  6901 This video features two presentations from a bootcamp on GPU progra…                   │
│   10. 0.857  2160 This talk delves into optimizing the performance of Reinforcement L…                   │
│   11. 0.857  6796 In this talk, Tri Dao, the inventor of FlashAttention, explores the…                   │
│   12. 0.856  6900 This video features two talks on advanced GPU programming for AI. W…                   │
│   13. 0.855 10184 As far as I can tell, this is a technical lecture by Polus, formerl…                   │
│   14. 0.855  6794 This presentation explores the landscape of Domain Specific Languag…                   │
│   15. 0.855  3224 This presentation delves into hardware-efficient training methodolo…                   │
│   16. 0.853  2152 This talk, part of the "GPU mode" series, features Vincent from the…                   │
│   17. 0.851   970 **INT8 Tensor Core Matmuls for Turing: An Educational Journey into …                   │
│   18. 0.851  8905 This video introduces Helium, a new Python-embedded Domain-Specific…                   │
│   19. 0.847     4 **Lecture 24: Scan at the Speed of Light - Optimizing Parallel Pref…                   │
└──────────────────────────────────────────────────────────────────────────── 20 groups (20 total results) ┘
┌ Selected Result Preview ─────────────────────────────────────────────────────────────────────────────────┐
│ID: 4653  Host: 193.8.40.111  Model: gemini-2.5-flash-lite| input-price: 0.1 output-price: 0.4 max-context│
│Tokens: In:42428 Out:1317  Finished: 2025-07-27T12:37:28.093822 (duration: 6s)                            │
│Link: https://www.youtube.com/watch?v=mF_H_JGOFAc                                                         │
│Summary:                                                                                                  │
│Abstract:                                                                                                 │
│                                                                                                          │
│This video features Ali Hassani discussing "neighborhood attention," a sparse attention mechanism for deep│
│learning models, and its implementation. The talk delves into the motivation for sparse attention due to  │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
 ↑↓ Navigate  Space Expand  Enter Detail  y Yank Link  o Open Link  Esc Back  Found 20 similar transcripts
```
