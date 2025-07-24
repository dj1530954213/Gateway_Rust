# 10 – CI/CD · OPS · Security 指南

## 1. Git Flow
* `feature/*` → PR → `dev` (CI fmt/clippy/test) → squash merge → `main` (docker build tag)  
* Commit message = Conventional Commits

## 2. GitHub Actions
* ci.yml：fmt + clippy + unit  
* docker-release.yml：on tag v* push multi‑arch image

## 3. Make Targets
setup / unit / bench / compose / lint‑schema / cargo‑deny

## 4. License Policy
仅 MIT 或 Apache‑2.0；CI `cargo-deny` check on PR

## 5. Threat Model (简)
| 资产 | 威胁 | 控制 |
| 驱动 .so | 木马 | ed25519 签名 + SHA256 白名单 |
| MQTT creds | 泄露 | env file + k8s secret mount |
| RocksDB WAL | 恶意篡改 | dir fs ACL + node sealing |

## 6. Incident Runbook
* FrameBus backlog_lag > 1e6 → 扩容 RingPow  
* cfg_apply_total{result="err"} > 0 → rollback