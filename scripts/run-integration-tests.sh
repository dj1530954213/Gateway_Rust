#!/bin/bash

# 集成测试运行脚本
# 用法: ./scripts/run-integration-tests.sh [test_name]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INTEGRATION_DIR="$PROJECT_ROOT/tests/integration"

echo "🚀 启动集成测试..."
echo "项目根目录: $PROJECT_ROOT"
echo "集成测试目录: $INTEGRATION_DIR"

# 检查依赖
check_dependencies() {
    echo "🔧 检查依赖..."
    
    if ! command -v docker &> /dev/null; then
        echo "❌ 错误: Docker 未安装"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        echo "❌ 错误: docker-compose 未安装"  
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        echo "❌ 错误: Docker 服务未运行"
        exit 1
    fi
    
    echo "✅ 依赖检查通过"
}

# 启动测试环境
setup_test_environment() {
    echo "🐳 启动测试环境..."
    
    cd "$INTEGRATION_DIR"
    
    # 停止可能存在的容器
    docker-compose down -v &> /dev/null || true
    
    # 启动测试服务
    docker-compose up -d
    
    echo "⏳ 等待服务启动..."
    sleep 15
    
    # 健康检查
    echo "🏥 检查服务健康状态..."
    
    # 检查MQTT
    if docker exec gateway-test-mqtt mosquitto_pub -h localhost -t test -m "test" &> /dev/null; then
        echo "  ✅ MQTT Broker 就绪"
    else
        echo "  ❌ MQTT Broker 未就绪"
        docker-compose logs mqtt-broker
        exit 1
    fi
    
    # 检查Mock PLC
    if curl -s http://localhost:8080/ &> /dev/null; then
        echo "  ✅ Mock PLC 就绪"
    else
        echo "  ❌ Mock PLC 未就绪"
        docker-compose logs mock-plc
        exit 1
    fi
    
    echo "✅ 测试环境启动完成"
}

# 运行测试
run_tests() {
    echo "🧪 运行集成测试..."
    
    cd "$PROJECT_ROOT"
    
    local test_filter=""
    if [ -n "$1" ]; then
        test_filter="-- $1"
        echo "🎯 运行特定测试: $1"
    fi
    
    # 设置环境变量
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    
    # 运行集成测试
    if cargo test --test integration_tests $test_filter -- --test-threads=1; then
        echo "✅ 集成测试通过"
        return 0
    else
        echo "❌ 集成测试失败"
        return 1
    fi
}

# 清理测试环境
cleanup_test_environment() {
    echo "🧹 清理测试环境..."
    
    cd "$INTEGRATION_DIR"
    docker-compose down -v
    
    echo "✅ 测试环境已清理"
}

# 显示测试日志
show_logs() {
    echo "📋 显示服务日志..."
    
    cd "$INTEGRATION_DIR"
    docker-compose logs --tail=50
}

# 主函数
main() {
    local command="${1:-run}"
    
    case "$command" in
        "run")
            check_dependencies
            setup_test_environment
            
            # 捕获退出信号，确保清理
            trap cleanup_test_environment EXIT
            
            if run_tests "${2}"; then
                echo "🎉 集成测试全部通过!"
                exit 0
            else
                echo "💥 集成测试失败，显示日志..."
                show_logs
                exit 1
            fi
            ;;
        "setup")
            check_dependencies
            setup_test_environment
            echo "🎯 测试环境已启动，可以手动运行测试"
            ;;
        "cleanup")
            cleanup_test_environment
            ;;
        "logs")
            show_logs
            ;;
        "help"|"-h"|"--help")
            echo "用法: $0 [命令] [测试名称]"
            echo ""
            echo "命令:"
            echo "  run [test]  - 运行集成测试 (默认)"
            echo "  setup       - 只启动测试环境"
            echo "  cleanup     - 清理测试环境"
            echo "  logs        - 显示服务日志"
            echo "  help        - 显示此帮助信息"
            echo ""
            echo "示例:"
            echo "  $0                                    # 运行所有集成测试"
            echo "  $0 run test_end_to_end_data_flow     # 运行特定测试"
            echo "  $0 setup                             # 只启动环境"
            echo "  $0 cleanup                           # 清理环境"
            ;;
        *)
            echo "❌ 未知命令: $command"
            echo "使用 '$0 help' 查看可用命令"
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"