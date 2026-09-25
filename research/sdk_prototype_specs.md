# HELIUM SDK: THE ONE-LINE INTEGRATION PROTOTYPE

**Status**: Prototype Spec v0.1
**Objective**: Reduce "Integration Load" from 1/5 to 4/5 on the scorecard.

## 1. INSTALLATION
Researchers can install Helium directly into their training environment:
```bash
pip install helium-sdk
```

## 2. THE "ONE-LINE" USAGE
The goal is to allow a researcher to use Helium as a "plugin" to their existing training loop (PyTorch/TensorFlow).

```python
import helium

# 1. Initialize with your Helium Wallet / API Key
client = helium.Client(api_key="HLM_GENESIS_...")

# 2. Wrap your training job
# This one command:
# - Deploys your model architecture to the Helium Forge
# - Syncs your dataset (or a pointer to it)
# - Starts the PoUW-verified training epoch
job = client.submit_training(
    model=my_pytorch_model,
    dataset="s3://my-data/dataset.tar.gz",
    target_metric={"accuracy": 0.98},
    max_budget_hlm=50
)

# 3. Monitor in real-time
print(f"Job Status: {job.status}") # 'Training on RTX 4090 @ Node_7...'
```

## 3. VERIFICATION INTEGRATION
After training, the SDK automatically fetches the ZK-Proof from the Helium L1.
```python
if job.verify():
    print("Proof of Useful Work VALIDATED.")
    final_weights = job.download_weights()
```

## 4. NEXT STEPS (MVP)
*   Create a Mock Python class to simulate this flow for Discord demos.
*   Integrate this flow into the "READ SPECS" section of the landing page.
