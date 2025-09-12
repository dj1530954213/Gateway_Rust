#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   bash scripts/run-webgw-api-dev.sh up         # start deps and web-gw-api
#   bash scripts/run-webgw-api-dev.sh up-no-deps # only start web-gw-api (assumes deps already up)
#   bash scripts/run-webgw-api-dev.sh logs       # tail logs
#   bash scripts/run-webgw-api-dev.sh down       # stop and remove container
#
# Ports
#   REST:    18080 (container 8080)
#   Metrics: 19090 (container 19090)
#
# Endpoints
#   http://localhost:18080/docs/openapi.json
#   http://localhost:18080/docs/swagger-ui/
#   http://localhost:18080/system/health
#   http://localhost:18080/api/v1/system/health

NAME=webgw-api-dev
HTTP_PORT=18080
METRICS_PORT=19090
RUST_IMAGE=rust:1.85-bullseye
CWD=$(cd "$(dirname "$0")/.." && pwd)
TARGET_DIR=/app/target-docker

function up_deps() {
  echo "[deps] starting docker-compose dependencies..."
  if command -v docker compose >/dev/null 2>&1; then
    (cd "$CWD" && docker compose up -d postgres nats emqx prometheus grafana influxdb)
  else
    (cd "$CWD" && docker-compose up -d postgres nats emqx prometheus grafana influxdb)
  fi
}

function up_webgw() {
  echo "[webgw] starting container $NAME on host network..."
  docker rm -f "$NAME" >/dev/null 2>&1 || true
  docker run -d --network host --name "$NAME" \
    -e RUST_LOG=${RUST_LOG:-info} \
    -e WEBGW_LOG_LEVEL=info \
    -e ENV=dev \
    -e WEBGW_HTTP_ADDR=0.0.0.0:$HTTP_PORT \
    -e CARGO_TARGET_DIR=$TARGET_DIR \
    -v "$CWD":/app -w /app \
    $RUST_IMAGE bash -lc \
    "apt-get update -qq >/dev/null && apt-get install -y -qq pkg-config libssl-dev clang libclang-dev >/dev/null && export PATH=/usr/local/cargo/bin:\$PATH; mkdir -p $TARGET_DIR; cargo clean -p web-gw-api || true; cargo run -p web-gw-api --bin web-gw-api"
}

function wait_ready() {
  echo "[probe] waiting for /docs/openapi.json (max 900s) and /system/health (max 300s)..."
  for i in $(seq 1 900); do
    code=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:$HTTP_PORT/docs/openapi.json || echo 000)
    if [ "$code" = "200" ]; then echo "openapi: $code"; break; fi; 
    if [ $((i % 15)) -eq 0 ]; then echo "[logs] tail while waiting (i=$i)"; docker logs --tail 30 "$NAME" | sed -n "1,60p"; fi; 
    sleep 1
  done
  for i in $(seq 1 300); do
    code=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:$HTTP_PORT/system/health || echo 000)
    if [ "$code" != "000" ]; then echo "sys_health: $code"; break; fi; 
    if [ $((i % 15)) -eq 0 ]; then echo "[logs] tail while waiting (i=$i)"; docker logs --tail 30 "$NAME" | sed -n "1,60p"; fi; 
    sleep 1
  done

  echo "[check] quick curl:"
  for p in /health /health/live /health/ready /system/health /api/v1/system/health /docs/openapi.json /docs/swagger-ui/; do
    printf "  %s -> " "$p"; curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:$HTTP_PORT$p || echo ERR
  done
}

case "${1:-up}" in
  up)
    up_deps
    up_webgw
    wait_ready
    ;;
  up-no-deps)
    up_webgw
    wait_ready
    ;;
  logs)
    docker logs -f "$NAME"
    ;;
  down)
    docker rm -f "$NAME" || true
    ;;
  *)
    echo "unknown cmd: $1"; exit 1
    ;;
 esac
