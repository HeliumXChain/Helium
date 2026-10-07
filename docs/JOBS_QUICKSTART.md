# Jobs Quickstart — Kuro x Helium Compute (Q5)

Run a job in 1 command. Everything below targets the local node
(`127.0.0.1:8787` + `helium` CLI). Remote access goes through the private
WireGuard mesh or an SSH tunnel — never expose the ports.

## 1. Manual job (no market)

```bash
helium workload run --template jupyter --name my-first-job
helium workload list
```

## 2. Market flow (credits + scheduler)

```bash
helium faucet --party me --amount 100
helium market offer --amount 32 --price 0.5
helium market request --template train --max-price 1.0 --hours 2
helium market match --request-id <req-id>
helium market accept --match-id <match-id>   # settles + records the workload
helium workload list
```

## 3. Via Kuro (any project)

```bash
curl -s -X POST 127.0.0.1:8767/api/compute/request \
  -H 'Content-Type: application/json' \
  -d '{"project":"openquant","template":"batch-scan","hours":1}'
```

## Templates

| name | image | gpus | for |
|---|---|---|---|
| `jupyter` | jupyter/pytorch-notebook:latest | 1 | JupyterLab :8888 |
| `train` | pytorch/pytorch:latest | 1 | training scripts |
| `batch-scan` | ubuntu:22.04 | 0 | CPU scans, ETL |

Explicit `--image` / `--gpus` (or API fields) override the template.
Unknown names fail with the known list — locally and over HTTP (400).

## Bench (score soutenu, jamais le pic)

```bash
helium bench --seconds 60            # humain
helium bench --seconds 300 --json    # score officiel (5 min, throttling visible)
helium market offer --amount 32 --price 0.5 --bench          # offre + score inline
helium market request --template train --min-bench 10        # filtre les offres < 10 GFLOPS
```

Scores relatifs (meme binaire, meme classe CPU), binaire `--release`
exige pour un score officiel. `derate_pct` = throttling thermique.

## Knobs (env)

| var | default | meaning |
|---|---|---|
| `HELIUM_MAX_ACTIVE_JOBS_PER_REQUESTER` | 2 | concurrent jobs per requester (429 beyond) |
| `HELIUM_REQUEST_TTL_HOURS` | 24 | open requests expire after this |
| `HELIUM_HOME` | ~/.helium | state dir (tests, portable nodes) |
| `HELIUM_API_URL` / `HELIUM_API_TOKEN` | 127.0.0.1:8787 | Kuro-side client config |

Shell note (Windows): run demos under Git-Bash/PowerShell — WSL interop
does not forward custom env to Windows .exe.
