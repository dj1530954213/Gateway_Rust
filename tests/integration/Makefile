# 集成测试环境管理 Makefile

.PHONY: help setup teardown test status logs check-deps

# 默认目标
help:
	@echo "集成测试环境管理"
	@echo ""
	@echo "可用命令:"
	@echo "  setup      - 启动测试环境"
	@echo "  teardown   - 停止并清理测试环境"
	@echo "  status     - 查看服务状态"
	@echo "  logs       - 查看服务日志"
	@echo "  test       - 运行集成测试"
	@echo "  check-deps - 检查依赖"

# 检查依赖
check-deps:
	@echo "检查依赖..."
	@which docker >/dev/null 2>&1 || (echo "错误: Docker 未安装" && exit 1)
	@which docker-compose >/dev/null 2>&1 || (echo "错误: docker-compose 未安装" && exit 1)
	@docker info >/dev/null 2>&1 || (echo "错误: Docker 服务未运行" && exit 1)
	@echo "✅ 依赖检查通过"

# 启动测试环境
setup: check-deps
	@echo "🚀 启动集成测试环境..."
	docker-compose up -d
	@echo "⏳ 等待服务启动..."
	@sleep 10
	@echo "✅ 测试环境启动完成"
	@make status

# 停止测试环境
teardown:
	@echo "🧹 停止集成测试环境..."
	docker-compose down -v
	@echo "✅ 测试环境已清理"

# 查看服务状态
status:
	@echo "📊 服务状态:"
	@docker-compose ps

# 查看日志
logs:
	@echo "📝 服务日志:"
	docker-compose logs --tail=50

# 查看特定服务日志
logs-mqtt:
	docker-compose logs -f mqtt-broker

logs-plc:
	docker-compose logs -f mock-plc

logs-influx:
	docker-compose logs -f influxdb

# 运行集成测试
test: setup
	@echo "🧪 运行集成测试..."
	cd ../.. && cargo test --test integration_tests -- --test-threads=1
	@echo "✅ 集成测试完成"

# 重启测试环境
restart: teardown setup

# 健康检查
health:
	@echo "🏥 健康检查:"
	@echo "MQTT Broker:"
	@docker exec gateway-test-mqtt mosquitto_pub -h localhost -t test -m "test" >/dev/null 2>&1 && echo "  ✅ MQTT OK" || echo "  ❌ MQTT Failed"
	@echo "InfluxDB:"
	@curl -s http://localhost:8086/ping >/dev/null 2>&1 && echo "  ✅ InfluxDB OK" || echo "  ❌ InfluxDB Failed"
	@echo "Mock PLC:"
	@curl -s http://localhost:8080/ >/dev/null 2>&1 && echo "  ✅ Mock PLC OK" || echo "  ❌ Mock PLC Failed"

# 清理 Docker 资源
clean: teardown
	@echo "🗑️  清理 Docker 资源..."
	docker system prune -f
	docker volume prune -f