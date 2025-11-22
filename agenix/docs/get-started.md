# Get Started with AGEniX

Welcome to AGEniX! This guide will walk you through installing and running your first agentic workflow in under 5 minutes.

## Installation

Install AGX, AGQ, and AGW with one command:

```bash
curl -fsSL https://agenix.sh/install.sh | bash
```

This installs all three components to `~/.local/bin`. Make sure this directory is in your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Verify installation:

```bash
agx --version  # Should show: agx 0.1.0
agq --version  # Should show: agq 0.1.0
agw --version  # Should show: agw 0.1.0
```

## Quick Start: Your First Workflow

### Step 1: Start the Queue Manager (AGQ)

AGQ is the central coordinator that manages jobs and workers. Start it first:

```bash
# Generate a secure session key
export AGQ_SESSION_KEY=$(openssl rand -hex 32)

# Start AGQ on localhost
agq --bind 127.0.0.1:6379 --session-key "$AGQ_SESSION_KEY"
```

You should see:

```
AGQ v0.1.0 starting...
Listening on 127.0.0.1:6379
Session key configured (64 hex chars)
Ready to accept connections
```

**Keep this terminal open** - AGQ needs to run continuously.

### Step 2: Start Workers (AGW)

Open a **new terminal** and start worker processes. Workers execute the actual tasks.

```bash
# Set connection details
export AGQ_ADDR="127.0.0.1:6379"
export AGQ_SESSION_KEY="<paste-your-session-key-here>"

# Start workers
agw &
agw &
```

Each worker will register with AGQ and wait for jobs.

> **Note**: Worker naming (`agw --name worker-1`) is coming in a future release. See [issue #26](https://github.com/agenix-sh/agw/issues/26). For now, workers auto-generate IDs.

### Step 3: Create and Execute a Plan (AGX)

Open a **third terminal** and use AGX to create an agentic plan.

```bash
# Set connection details
export AGQ_ADDR="127.0.0.1:6379"
export AGQ_SESSION_KEY="<paste-your-session-key-here>"

# Start AGX REPL
agx
```

You'll see the AGX prompt:

```
AGX v0.1.0 - Agentic Execution Planner
Type 'help' for commands, 'quit' to exit

agx>
```

### Step 4: Build Your First Plan

In the AGX REPL, create a simple plan with a few tasks:

```
agx> new
Plan created (empty)

agx> add "echo 'Step 1: Initializing system'"
Task added: echo 'Step 1: Initializing system'

agx> add "sleep 2"
Task added: sleep 2

agx> add "echo 'Step 2: Processing data'"
Task added: echo 'Step 2: Processing data'

agx> add "date"
Task added: date

agx> add "echo 'Step 3: Complete!'"
Task added: echo 'Step 3: Complete!'
```

### Step 5: Review Your Plan

Check what you've created:

```
agx> show
Plan (5 tasks):
  1. echo 'Step 1: Initializing system'
  2. sleep 2
  3. echo 'Step 2: Processing data'
  4. date
  5. echo 'Step 3: Complete!'
```

### Step 6: Submit to the Queue

Send your plan to AGQ for execution:

```
agx> submit
Plan submitted to queue
Job ID: job_abc123def456
```

The workers will immediately start processing tasks from your plan!

### Step 7: Monitor Progress

Use the operational commands to monitor execution:

#### Check Queue Statistics

```
agx> stats
Queue Statistics:
  Total jobs: 1
  Active jobs: 1
  Pending jobs: 0
  Completed jobs: 0
  Total tasks processed: 3
  Tasks in queue: 2
```

#### List Active Jobs

```
agx> j
Active Jobs:
  job_abc123def456
    Status: running
    Progress: 3/5 tasks complete
    Worker: worker-1
```

#### List Active Workers

```
agx> w
Active Workers:
  worker-1
    Status: busy
    Current job: job_abc123def456
    Tasks completed: 15
  worker-2
    Status: idle
    Tasks completed: 8
```

### Step 8: Check Results

After a few seconds, check stats again:

```
agx> stats
Queue Statistics:
  Total jobs: 1
  Active jobs: 0
  Pending jobs: 0
  Completed jobs: 1
  Total tasks processed: 5
  Tasks in queue: 0
```

Your workflow is complete! 🎉

## Understanding the Architecture

### The Five Execution Layers

AGEniX uses a hierarchical execution model:

1. **Task** - High-level user request ("analyze this document")
2. **Plan** - Sequence of actions to accomplish the task
3. **Job** - Plan submitted to the queue for execution
4. **Action** - Individual executable step (bash command, tool call)
5. **Workflow** - Coordinated execution across workers

### Component Roles

- **AGX** (Planner) - Creates plans from tasks, submits to queue
- **AGQ** (Queue) - Manages jobs, distributes work to workers
- **AGW** (Worker) - Executes actions and reports results

### Communication Flow

```
┌─────┐                    ┌─────┐                    ┌─────┐
│ AGX │ ──── submit ────> │ AGQ │ ──── assign ────> │ AGW │
│     │ <─── status ────  │     │ <─── results ───  │     │
└─────┘                    └─────┘                    └─────┘
  Plan                      Job                       Action
  Creator                   Manager                   Executor
```

All communication uses the RESP protocol (Redis Serialization Protocol) over TCP.

## REPL Commands Reference

### Plan Management
- `new` - Create a new empty plan
- `add "<command>"` - Add a task to the current plan
- `show` - Display the current plan
- `clear` - Clear the current plan
- `submit` - Submit plan to AGQ as a job

### Operational Visibility (Monitoring)
- `stats` - Show queue statistics (QUEUE.STATS)
- `j` - List active jobs (JOBS.LIST)
- `w` - List active workers (WORKERS.LIST)

### System
- `help` - Show all available commands
- `quit` - Exit the REPL

## Common Workflows

### Example 1: Data Processing Pipeline

```bash
agx> new
agx> add "curl -o data.json https://api.example.com/data"
agx> add "jq '.records[]' data.json > records.jsonl"
agx> add "wc -l records.jsonl"
agx> add "rm data.json records.jsonl"
agx> submit
```

### Example 2: Multi-Step Build

```bash
agx> new
agx> add "git clone https://github.com/user/repo.git"
agx> add "cd repo && cargo build --release"
agx> add "cd repo && cargo test"
agx> add "cp repo/target/release/app /usr/local/bin/"
agx> submit
```

### Example 3: System Maintenance

```bash
agx> new
agx> add "apt-get update"
agx> add "apt-get upgrade -y"
agx> add "apt-get autoremove -y"
agx> add "systemctl restart nginx"
agx> submit
```

## Next Steps

### Run Multiple Plans Concurrently

AGQ can handle multiple jobs simultaneously. Just create and submit multiple plans:

```bash
# Terminal 1 (AGX instance 1)
agx> new
agx> add "process-dataset-1.sh"
agx> submit

# Terminal 2 (AGX instance 2)
agx> new
agx> add "process-dataset-2.sh"
agx> submit
```

Both jobs will run in parallel across your worker pool.

### Scale Workers Dynamically

Add more workers anytime:

```bash
# Start additional workers
agw &
agw &
agw &
```

AGQ automatically distributes work to all available workers.

### Monitor from Any AGX Instance

You can run `agx` from multiple terminals and use operational commands (`stats`, `j`, `w`) to monitor the same queue:

```bash
# Terminal 1
agx> stats

# Terminal 2
agx> j

# Terminal 3
agx> w
```

All instances see the same queue state.

## Troubleshooting

### Connection Refused

**Problem:** AGX cannot connect to AGQ

**Solution:**
1. Verify AGQ is running: `lsof -i :6379`
2. Check `AGQ_ADDR` matches AGQ bind address
3. Verify `AGQ_SESSION_KEY` matches on both sides

### Authentication Failed

**Problem:** "Invalid session key" error

**Solution:**
- Ensure `AGQ_SESSION_KEY` environment variable is set correctly
- The key must be exactly 64 hexadecimal characters
- Use the same key for AGQ, AGX, and AGW

### Workers Not Picking Up Jobs

**Problem:** Job submitted but no workers process it

**Solution:**
1. Check workers are running: `agx> w`
2. Verify workers have correct `AGQ_ADDR` and `AGQ_SESSION_KEY`
3. Restart workers if needed

### Command Not Found

**Problem:** `agx: command not found`

**Solution:**
Add `~/.local/bin` to PATH:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Learn More

- **Architecture** - [Execution Layers](/docs/architecture/execution-layers)
- **Security** - [Zero-Trust Execution](/docs/zero-trust/zero-trust-execution)
- **API Reference** - [RESP Protocol](/docs/api/resp-protocol)
- **Advanced Usage** - [Agentic Units](/docs/au-specs/agentic-unit-spec)

## Getting Help

- **Documentation:** https://agenix.sh/docs
- **GitHub Issues:** https://github.com/agenix-sh/agenix/issues
- **Examples:** https://github.com/agenix-sh/agenix/tree/main/examples

---

**Ready to build more complex workflows?** Check out the [Agentic Units guide](/docs/au-specs/agentic-unit-spec) to learn how to integrate specialized AI tools into your plans.
