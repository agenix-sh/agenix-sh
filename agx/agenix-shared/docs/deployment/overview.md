# Deployment Overview

This document outlines deployment patterns for the AGEnix ecosystem.

## Single-Node (Local) Development

- Install `agx` and AU tools (e.g. `agx-ocr`) on a single machine.
- Use local-mode execution (`agx` runs the plan directly).
- Ideal for prototyping and small workflows.

## Distributed Execution (AGQ + AGW)

- Run `agq` connected to a queue backend (e.g. Redis).
- Run one or more `agw` workers on the same or different machines.
- Configure `agx` to submit jobs to `agq` rather than executing locally.
- Use this mode for embarrassingly parallel workloads (e.g. OCR across many files).

Future documents will cover containerisation, orchestration (e.g. Docker Compose, Kubernetes), and observability.
