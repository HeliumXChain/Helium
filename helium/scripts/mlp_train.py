#!/usr/bin/env python3
"""Micro language model (char MLP, manual backprop) — Helium training demo.

Runs inside the provider microVM: proves borrower workloads train in
provider RAM. Same loop as big transformers, tiny scale for CPU demo:
embedding + tanh MLP + cross-entropy, ~50k params.
Usage: python3 mlp_train.py [--iters 800]
"""
import sys
import numpy as np

TEXT = (
    """
partager le gpu entre amis de confiance est simple avec helium.
le tunnel chiffre protege les donnees pendant le calcul.
la microvm isolee execute le workload en toute securite.
les credits locaux paient les ressources sans blockchain.
le borrower emprunte la ram du provider pour entrainer son modele.
le provider prete son gpu quand il est inutilise la nuit.
le reseau prive decouvre les voisins sans serveur central.
le matching choisit la meilleure offre au meilleur prix.
le daemon regle les credits tout seul toutes les trente secondes.
le dashboard affiche le noeud le marche et le tunnel.
"""
    * 6
)

BLOCK = 3
N_EMBD = 30
N_HIDDEN = 100
ITERS = int(sys.argv[sys.argv.index("--iters") + 1]) if "--iters" in sys.argv else 800
BATCH = 64
LR = 0.05

chars = sorted(set(TEXT))
stoi = {c: i for i, c in enumerate(chars)}
itos = {i: c for c, i in stoi.items()}
V = len(chars)
data = np.array([stoi[c] for c in TEXT], dtype=np.int64)
print(f"corpus={len(data)} chars vocab={V}", flush=True)

rng = np.random.default_rng(0)
C = rng.standard_normal((V, N_EMBD)) * 0.5
W1 = rng.standard_normal((BLOCK * N_EMBD, N_HIDDEN)) * (5 / 3) / (BLOCK * N_EMBD) ** 0.5
b1 = np.zeros(N_HIDDEN)
W2 = rng.standard_normal((N_HIDDEN, V)) * 0.01
b2 = np.zeros(V)
params = [C, W1, b1, W2, b2]


def forward(ix):
    emb = C[ix]  # (B,T,C)
    x = emb.reshape(ix.shape[0], -1)  # (B,T*C)
    hpre = x @ W1 + b1
    h = np.tanh(hpre)
    logits = h @ W2 + b2
    return emb, x, hpre, h, logits


def loss_and_grad(ix, iy):
    B = ix.shape[0]
    emb, x, hpre, h, logits = forward(ix)
    logits = logits - logits.max(axis=1, keepdims=True)
    probs = np.exp(logits)
    probs = probs / probs.sum(axis=1, keepdims=True)
    loss = -np.log(probs[np.arange(B), iy] + 1e-12).mean()
    dlogits = probs.copy()
    dlogits[np.arange(B), iy] -= 1
    dlogits /= B
    dW2 = h.T @ dlogits
    db2 = dlogits.sum(axis=0)
    dh = dlogits @ W2.T
    dhpre = dh * (1 - h**2)
    dW1 = x.T @ dhpre
    db1 = dhpre.sum(axis=0)
    dx = dhpre @ W1.T
    demb = dx.reshape(emb.shape)
    dC = np.zeros_like(C)
    np.add.at(dC, ix, demb)
    return loss, [dC, dW1, db1, dW2, db2]


for it in range(1, ITERS + 1):
    i = rng.integers(0, len(data) - BLOCK - 1, size=BATCH)
    ix = np.stack([data[j : j + BLOCK] for j in i])
    iy = np.array([data[j + BLOCK] for j in i])
    loss, grads = loss_and_grad(ix, iy)
    for p, g in zip(params, grads):
        p -= LR * g
    if it % 100 == 0 or it == 1:
        print(f"iter {it}/{ITERS} loss={loss:.3f}", flush=True)

rng2 = np.random.default_rng(7)
ctx = [0] * BLOCK
out = []
for _ in range(120):
    _, _, _, _, logits = forward(np.array([ctx]))
    probs = np.exp(logits[0] - logits[0].max())
    probs /= probs.sum()
    nxt = int(rng2.choice(V, p=probs))
    out.append(itos[nxt])
    ctx = ctx[1:] + [nxt]
print("SAMPLE:" + "".join(out), flush=True)
print("TRAIN-DONE", flush=True)
