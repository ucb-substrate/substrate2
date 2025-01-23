---
sidebar_position: 0
slug: /
---

# Introduction

Substrate is a suite of Rust libraries for generating analog/mixed-signal integrated circuits (ICs).
Among other things, it provides:

- Parametric layout/schematic generation
- Simulation waveform verification
- Waveform query utilities
- Higher-level layout APIs (grid/track-based, tiling)
- Full control over exported collateral via intermediate representations (IRs)
- Rudimentary caching, multi-threading, error handling
- An ecosystem of simulation, DRC, LVS, and extraction plugins

The primary goal of Substrate is to enable writing generators, or programs that make circuits from a set of specifications.
