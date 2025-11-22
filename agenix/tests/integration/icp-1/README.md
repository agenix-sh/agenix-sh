# ICP-1 Integration Tests

Integration tests for validating end-to-end job execution across AGX, AGQ, and AGW.

## Prerequisites

Build all components in release mode:

```bash
cd /Users/lewis/work/agenix-sh/agx && cargo build --release
cd /Users/lewis/work/agenix-sh/agq && cargo build --release
cd /Users/lewis/work/agenix-sh/agw && cargo build --release
```

## Test Structure

```
icp-1/
├── README.md                 # This file
├── run_all.sh                # Run all ICP-1 tests
├── test_1_simple_job.sh      # Single-task job execution
├── test_2_pipeline.sh        # Multi-task pipeline with piping
├── test_3_worker_lifecycle.sh # Worker heartbeat and cleanup
├── test_4_error_handling.sh  # Error scenarios (timeout, invalid cmd)
├── test_5_status_query.sh    # Job status and output retrieval
├── fixtures/
│   ├── simple-job.json       # Single echo command
│   ├── pipeline-job.json     # Sort + uniq pipeline
│   ├── timeout-job.json      # Job that exceeds timeout
│   ├── invalid-job.json      # Job with non-existent command
│   └── sample-input.txt      # Test data for pipeline
└── helpers/
    ├── config.sh             # Shared configuration
    ├── start_agq.sh          # Start AGQ in background
    ├── start_agw.sh          # Start AGW in background
    ├── cleanup.sh            # Kill all test processes
    └── wait_for_status.sh    # Poll job status until complete
```

## Running Tests

### Run all tests
```bash
./run_all.sh
```

### Run individual test
```bash
./test_1_simple_job.sh
```

### Cleanup after failed test
```bash
./helpers/cleanup.sh
```

## Environment Variables

- `AGX_BIN` - Path to agx binary (default: agx repo release build)
- `AGQ_BIN` - Path to agq binary (default: agq repo release build)
- `AGW_BIN` - Path to agw binary (default: agw repo release build)
- `TEST_PORT` - AGQ port (default: 6379)
- `TEST_TIMEOUT` - Max wait for job completion (default: 30s)

## Exit Codes

- `0` - All tests passed
- `1` - Test failed
- `2` - Test setup failed (missing binaries, port in use)
- `3` - Test timeout

## Troubleshooting

### Port already in use
```bash
lsof -ti:6379 | xargs kill -9
```

### Stale test processes
```bash
./helpers/cleanup.sh
```

### View component logs
```bash
tail -f /tmp/agq-test.log
tail -f /tmp/agw-test.log
```
