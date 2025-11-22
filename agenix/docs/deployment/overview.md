# Deployment Overview

**Version:** 1.0
**Status:** Deployment Patterns Guide
**Last Updated:** 2025-11-17

This document outlines deployment patterns for the AGEniX ecosystem across different scales and use cases.

---

## Table of Contents

1. [Deployment Modes](#deployment-modes)
2. [Single-Node (Local) Development](#single-node-local-development)
3. [Distributed Execution](#distributed-execution)
4. [Container Deployment](#container-deployment)
5. [Kubernetes Deployment](#kubernetes-deployment)
6. [Configuration Management](#configuration-management)
7. [Observability](#observability)

---

## 1. Deployment Modes

### Mode Comparison

| Mode | Components | Use Case | Scalability | Complexity |
|------|-----------|----------|-------------|------------|
| **Local** | AGX only | Development, prototyping | Single machine | Low |
| **Local + Workers** | AGX + AGW | Parallel processing on one machine | CPU cores | Low |
| **Distributed** | AGX + AGQ + AGW | Production, large workloads | Many machines | Medium |
| **Kubernetes** | All components | Enterprise, auto-scaling | Cloud-scale | High |

---

## 2. Single-Node (Local) Development

### Overview

Ideal for:
- Prototyping and experimentation
- Small workflows (single files, quick tasks)
- Development and testing
- Learning AGEniX

### Architecture

```
┌─────────────────────────────┐
│   Single Machine            │
│                             │
│  ┌──────┐                   │
│  │ AGX  │                   │
│  └──┬───┘                   │
│     │ Executes locally      │
│     ▼                       │
│  ┌─────────┐                │
│  │ AU Tools│                │
│  └─────────┘                │
│                             │
└─────────────────────────────┘
```

### Installation

```bash
# Install AGX
cargo install agx

# Install AU tools
cargo install agx-ocr

# Verify installation
agx --version
agx-ocr --version
```

### Usage

```bash
# Execute plan locally (no AGQ/AGW needed)
echo "data.txt" | agx "Extract text from images" --execute
```

### Limitations

- Single-threaded execution
- No job queuing
- No distributed processing
- Plans execute immediately (no scheduling)

---

## 3. Distributed Execution

### 3.1 Overview

Ideal for:
- Batch processing (100s-1000s of files)
- Parallel workloads (embarrassingly parallel tasks)
- Multiple workers on different machines
- Production deployments

### 3.2 Architecture

```
┌──────────────┐
│   Client     │
│  ┌──────┐    │
│  │ AGX  │    │
│  └───┬──┘    │
└──────┼───────┘
       │ Submit plan
       │
       ▼
┌────────────────────────────┐
│   Queue Manager            │
│  ┌────────┐                │
│  │  AGQ   │                │
│  │ (redb) │                │
│  └────┬───┘                │
└───────┼────────────────────┘
        │ Jobs
        │
    ┌───┴────┬────────┬────────┐
    │        │        │        │
    ▼        ▼        ▼        ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│ Worker │ │ Worker │ │ Worker │ │ Worker │
│  AGW   │ │  AGW   │ │  AGW   │ │  AGW   │
└────────┘ └────────┘ └────────┘ └────────┘
 Machine 1  Machine 2  Machine 3  Machine 4
```

### 3.3 Setup

**On Queue Manager Machine:**
```bash
# Install AGQ
cargo install agq

# Generate session key for workers
SESSION_KEY=$(openssl rand -hex 32)
echo "Worker session key: $SESSION_KEY"

# Start AGQ
agq --listen 0.0.0.0:6380 --session-key "$SESSION_KEY"
```

**On Worker Machines:**
```bash
# Install AGW
cargo install agw

# Install AU tools
cargo install agx-ocr

# Set session key (use same key from AGQ)
export AGW_SESSION_KEY="<session-key-from-agq>"

# Connect to AGQ and start worker
agw --agq-addr <agq-ip>:6380 --worker-id worker-$(hostname)-$$
```

**On Client Machine:**
```bash
# Install AGX
cargo install agx

# Configure AGQ endpoint
export AGX_AGQ_ADDR="<agq-ip>:6380"

# Submit plan to AGQ
echo "image1.png,image2.png,image3.png" | agx "Extract text" --submit
```

### 3.4 Scaling

**Add more workers:**
```bash
# On each new machine
agw --agq-addr <agq-ip>:6380 --worker-id worker-$(hostname)-$$
```

**Worker pools by capability:**
```bash
# GPU-enabled workers
agw --agq-addr <agq-ip>:6380 --worker-id gpu-worker-1 --tags gpu=true

# CPU-only workers
agw --agq-addr <agq-ip>:6380 --worker-id cpu-worker-1 --tags gpu=false
```

---

## 4. Container Deployment

### 4.1 Docker Compose

**Use case**: Multi-container local development or small production

**`docker-compose.yml`:**
```yaml
version: '3.8'

services:
  agq:
    image: agenix/agq:latest
    ports:
      - "6380:6380"
    environment:
      - AGQ_SESSION_KEY=${AGQ_SESSION_KEY}
    volumes:
      - agq-data:/data
    command: --listen 0.0.0.0:6380 --data-dir /data

  worker-1:
    image: agenix/agw:latest
    depends_on:
      - agq
    environment:
      - AGW_SESSION_KEY=${AGQ_SESSION_KEY}
      - AGW_AGQ_ADDR=agq:6380
    volumes:
      - ./workspace:/workspace
    command: --worker-id worker-1 --max-jobs 4

  worker-2:
    image: agenix/agw:latest
    depends_on:
      - agq
    environment:
      - AGW_SESSION_KEY=${AGQ_SESSION_KEY}
      - AGW_AGQ_ADDR=agq:6380
    volumes:
      - ./workspace:/workspace
    command: --worker-id worker-2 --max-jobs 4

volumes:
  agq-data:
```

**Usage:**
```bash
# Generate session key
export AGQ_SESSION_KEY=$(openssl rand -hex 32)

# Start services
docker-compose up -d

# Scale workers
docker-compose up -d --scale worker=10

# Submit job (from client)
docker run --rm -i agenix/agx:latest \
  --agq-addr <host-ip>:6380 \
  "Extract text" < inputs.txt
```

### 4.2 Dockerfiles

**AGQ Dockerfile:**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin agq

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/agq /usr/local/bin/
EXPOSE 6380
ENTRYPOINT ["/usr/local/bin/agq"]
```

**AGW Dockerfile:**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin agw

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    coreutils \
    grep \
    jq \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/agw /usr/local/bin/
# Install AUs
COPY --from=builder /build/target/release/agx-ocr /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/agw"]
```

---

## 5. Kubernetes Deployment

### 5.1 Overview

**Use case**: Production, auto-scaling, multi-tenant

**Architecture:**
```
┌────────────────────────────────────────┐
│         Kubernetes Cluster             │
│                                        │
│  ┌──────────────────────────────────┐  │
│  │   Namespace: agenix              │  │
│  │                                  │  │
│  │  ┌────────────┐                  │  │
│  │  │ AGQ        │                  │  │
│  │  │ Deployment │                  │  │
│  │  │ (1 replica)│                  │  │
│  │  └──────┬─────┘                  │  │
│  │         │ Service: agq:6380      │  │
│  │         │                        │  │
│  │  ┌──────▼──────┐                 │  │
│  │  │ AGW         │                 │  │
│  │  │ Deployment  │                 │  │
│  │  │ (auto-scale)│                 │  │
│  │  │ 2-20 replicas│                │  │
│  │  └─────────────┘                 │  │
│  │                                  │  │
│  │  PersistentVolumeClaim (AGQ data)│  │
│  └──────────────────────────────────┘  │
└────────────────────────────────────────┘
```

### 5.2 Kubernetes Manifests

**Namespace:**
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: agenix
```

**ConfigMap (Session Key):**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: agq-session-key
  namespace: agenix
type: Opaque
stringData:
  session-key: "<generated-session-key>"
```

**AGQ Deployment:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agq
  namespace: agenix
spec:
  replicas: 1
  selector:
    matchLabels:
      app: agq
  template:
    metadata:
      labels:
        app: agq
    spec:
      containers:
      - name: agq
        image: agenix/agq:latest
        ports:
        - containerPort: 6380
        env:
        - name: AGQ_SESSION_KEY
          valueFrom:
            secretKeyRef:
              name: agq-session-key
              key: session-key
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: agq-data
---
apiVersion: v1
kind: Service
metadata:
  name: agq
  namespace: agenix
spec:
  selector:
    app: agq
  ports:
  - port: 6380
    targetPort: 6380
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: agq-data
  namespace: agenix
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

**AGW Deployment (Auto-scaling):**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agw
  namespace: agenix
spec:
  replicas: 2
  selector:
    matchLabels:
      app: agw
  template:
    metadata:
      labels:
        app: agw
    spec:
      containers:
      - name: agw
        image: agenix/agw:latest
        env:
        - name: AGW_SESSION_KEY
          valueFrom:
            secretKeyRef:
              name: agq-session-key
              key: session-key
        - name: AGW_AGQ_ADDR
          value: "agq:6380"
        - name: AGW_WORKER_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2000m"
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: agw-hpa
  namespace: agenix
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: agw
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### 5.3 Deploy

```bash
# Create namespace
kubectl apply -f namespace.yaml

# Create session key secret
SESSION_KEY=$(openssl rand -hex 32)
kubectl create secret generic agq-session-key \
  --from-literal=session-key="$SESSION_KEY" \
  -n agenix

# Deploy AGQ
kubectl apply -f agq-deployment.yaml

# Deploy AGW workers
kubectl apply -f agw-deployment.yaml

# Verify deployment
kubectl get pods -n agenix
kubectl logs -n agenix deployment/agq
```

---

## 6. Configuration Management

### 6.1 Environment Variables

**AGQ:**
```bash
AGQ_LISTEN_ADDR="0.0.0.0:6380"      # Listen address
AGQ_SESSION_KEY="<session-key>"     # Authentication key
AGQ_DATA_DIR="/data"                # Database location
AGQ_MAX_CONNECTIONS="100"           # Max concurrent connections
AGQ_LOG_LEVEL="info"                # Logging level
```

**AGW:**
```bash
AGW_AGQ_ADDR="agq:6380"             # AGQ server address
AGW_SESSION_KEY="<session-key>"     # Authentication key
AGW_WORKER_ID="worker-001"          # Unique worker identifier
AGW_MAX_JOBS="4"                    # Max concurrent jobs
AGW_HEARTBEAT_INTERVAL="30"         # Heartbeat seconds
AGW_LOG_LEVEL="info"                # Logging level
```

**AGX:**
```bash
AGX_AGQ_ADDR="agq:6380"             # AGQ server address (if using distributed mode)
AGX_SESSION_KEY="<session-key>"     # Authentication key
AGX_PLANNER_MODEL="echo-default"    # Echo model selection
AGX_LOG_LEVEL="info"                # Logging level
```

### 6.2 Configuration Files

**agq.toml:**
```toml
[server]
listen_addr = "0.0.0.0:6380"
max_connections = 100

[storage]
data_dir = "/data"
ttl_cleanup_interval = 30  # seconds

[auth]
session_key_file = "/etc/agq/session_key"

[logging]
level = "info"
format = "json"
```

---

## 7. Observability

### 7.1 Metrics (Phase 3)

**Prometheus scrape config:**
```yaml
scrape_configs:
  - job_name: 'agq'
    static_configs:
    - targets: ['agq:6380']
    metrics_path: '/metrics'

  - job_name: 'agw'
    kubernetes_sd_configs:
    - role: pod
      namespaces:
        names:
        - agenix
    relabel_configs:
    - source_labels: [__meta_kubernetes_pod_label_app]
      action: keep
      regex: agw
```

**Key metrics:**
```
agq_jobs_total{status="completed"}
agq_jobs_total{status="failed"}
agq_workers_active
agq_queue_length{queue="ready"}
agw_jobs_active
agw_jobs_completed_total
agw_task_duration_seconds
```

### 7.2 Logging

**Structured logging (JSON):**
```json
{
  "timestamp": "2025-11-17T10:00:00Z",
  "level": "INFO",
  "component": "agq",
  "message": "Worker registered",
  "worker_id": "worker-001",
  "capabilities": ["agx-ocr", "grep", "jq"]
}
```

**Aggregation (Loki/Elasticsearch):**
```yaml
# Promtail config for AGQ logs
scrape_configs:
  - job_name: agq
    static_configs:
    - targets:
      - localhost
      labels:
        job: agq
        __path__: /var/log/agq/*.log
```

### 7.3 Tracing (Phase 4)

**OpenTelemetry integration:**
- Trace job lifecycle: submit → queue → pull → execute → complete
- Span per task execution
- Distributed tracing across AGX → AGQ → AGW

---

## Deployment Checklist

### Pre-Deployment

- [ ] Generate session keys (32+ bytes, cryptographically random)
- [ ] Plan worker capacity (estimate jobs/second, tasks/job)
- [ ] Configure storage (AGQ database persistence)
- [ ] Set resource limits (CPU, memory per component)
- [ ] Review security settings (listen address, TLS, network policies)

### Deployment

- [ ] Deploy AGQ (single instance for Phase 1)
- [ ] Verify AGQ is accessible (telnet/nc to port 6380)
- [ ] Deploy workers (start with 2-4, scale as needed)
- [ ] Verify workers register with AGQ
- [ ] Test with simple job (e.g., echo hello)
- [ ] Monitor logs for errors

### Post-Deployment

- [ ] Configure monitoring (metrics, logs)
- [ ] Set up alerting (worker failures, queue depth)
- [ ] Document deployment (architecture diagram, runbook)
- [ ] Test failover scenarios
- [ ] Establish backup strategy (AGQ database)

---

## Related Documentation

- [System Overview](../architecture/system-overview.md) - Architecture overview
- [Worker Registration](../api/worker-registration.md) - Worker lifecycle
- [Zero-Trust Execution](../zero-trust/zero-trust-execution.md) - Security model

---

**Maintained by:** AGX Core Team
**Review cycle:** Per release or on infrastructure changes
**Last deployment guide update:** 2025-11-17
