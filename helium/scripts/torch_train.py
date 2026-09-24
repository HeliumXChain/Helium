#!/usr/bin/env python3
"""Tiny PyTorch training demo — Helium provider microVM workload.

Proves real ML frameworks train inside the provider guest:
synthetic regression, small MLP, CPU only. No downloads needed.
Usage: python3 torch_train.py [--iters 200]
"""
import sys
import time

import torch

torch.set_num_threads(2)
torch.manual_seed(0)

ITERS = int(sys.argv[sys.argv.index("--iters") + 1]) if "--iters" in sys.argv else 200

# Synthetic task: y = 3*x0 - 2*x1 + noise
N, D = 512, 8
X = torch.randn(N, D)
true_w = torch.tensor([3.0, -2.0, 0.5, 0.0, 1.0, -1.0, 0.0, 0.25])
y = X @ true_w + 0.1 * torch.randn(N)

model = torch.nn.Sequential(
    torch.nn.Linear(D, 32),
    torch.nn.Tanh(),
    torch.nn.Linear(32, 1),
)
opt = torch.optim.Adam(model.parameters(), lr=0.05)
loss_fn = torch.nn.MSELoss()

t0 = time.time()
for it in range(1, ITERS + 1):
    opt.zero_grad()
    loss = loss_fn(model(X).squeeze(), y)
    loss.backward()
    opt.step()
    if it % 50 == 0 or it == 1:
        print(f"iter {it}/{ITERS} loss={loss.item():.4f}", flush=True)

print(f"DONE in {time.time() - t0:.1f}s", flush=True)
with torch.no_grad():
    err = (model(X).squeeze() - y).pow(2).mean().sqrt().item()
print(f"final RMSE={err:.4f} (noise floor ~0.10)", flush=True)
