# Migration to AMD64 & Tinker Model Setup

## Summary of Changes
- **Restored Scripts**: Updated `scripts/prepare_local_model.py` to be more robust and use a config file.
- **Configuration**: Created `tinker_config.json` with the discovered Model ID (`tinker://4891cc84-76cb-5204-a924-02ab64169ad0:train:0/weights/final`).
- **Git**: Committed these changes to the local branch.

## Next Steps on AMD64 Machine

1.  **Sync Code**:
    ```bash
    git pull origin master
    ```

2.  **Install Dependencies**:
    Ensure you have the `tinker` CLI and other dependencies installed.
    ```bash
    pip install -r requirements.txt # or similar
    ```

3.  **Run Model Preparation**:
    This script will attempt to download the adapter and merge it with the base model.
    ```bash
    # Ensure TINKER_API_KEY is set
    export TINKER_API_KEY=your_key_here
    python3 scripts/prepare_local_model.py
    ```

4.  **Troubleshooting**:
    If you see the "not a sampler weights checkpoint" error:
    - It might be a specific issue with how `tinker` handles this checkpoint type.
    - Try running `tinker run list` on that machine to see if the checkpoint status is different.

## Verification
After running the script, check for:
- `models/agenix-echo-v1-merged/config.json`
- `models/agenix-echo-v1-merged/model.safetensors` (or similar)
