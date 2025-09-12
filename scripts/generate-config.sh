#!/bin/bash
# 配置生成脚本
# 根据环境和参数自动生成配置文件

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_DIR="$PROJECT_DIR/config"
TEMPLATES_DIR="$CONFIG_DIR/templates"

# 默认参数
ENVIRONMENT="dev"
OUTPUT_DIR="$CONFIG_DIR"
GATEWAY_ID=""
GATEWAY_NAME=""
GATEWAY_LOCATION=""
INTERACTIVE=false
OVERWRITE=false

# 显示帮助信息
show_help() {
    cat << EOF
配置生成脚本

用法: $0 [选项]

选项:
  -e, --env ENV              环境类型 (dev/prod/edge, 默认: dev)
  -o, --output DIR           输出目录 (默认: config/)
  -i, --interactive          交互式配置
  --gateway-id ID            网关ID
  --gateway-name NAME        网关名称  
  --gateway-location LOC     网关位置
  --overwrite               覆盖已存在的文件
  -h, --help                显示此帮助信息

环境类型:
  dev   - 开发环境配置 (简化配置，本地服务)
  prod  - 生产环境配置 (完整功能，外部服务)
  edge  - 边缘环境配置 (资源受限，离线支持)

示例:
  $0 --env dev --interactive
  $0 --env prod --gateway-id gw001 --gateway-name "生产线网关" --gateway-location "车间A"
  $0 --env edge --output ./my-config/
EOF
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -i|--interactive)
            INTERACTIVE=true
            shift
            ;;
        --gateway-id)
            GATEWAY_ID="$2"
            shift 2
            ;;
        --gateway-name)
            GATEWAY_NAME="$2"
            shift 2
            ;;
        --gateway-location)
            GATEWAY_LOCATION="$2"
            shift 2
            ;;
        --overwrite)
            OVERWRITE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            echo "使用 --help 查看帮助信息"
            exit 1
            ;;
    esac
done

# 验证环境类型
if [[ ! "$ENVIRONMENT" =~ ^(dev|prod|edge)$ ]]; then
    echo "错误: 不支持的环境类型 '$ENVIRONMENT'"
    echo "支持的环境类型: dev, prod, edge"
    exit 1
fi

# 确保模板目录存在
if [[ ! -d "$TEMPLATES_DIR" ]]; then
    echo "错误: 找不到模板目录 $TEMPLATES_DIR"
    exit 1
fi

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

echo "🚀 开始生成 $ENVIRONMENT 环境配置..."
echo "📁 输出目录: $OUTPUT_DIR"

# 交互式配置
if [[ "$INTERACTIVE" == "true" ]]; then
    echo ""
    echo "📝 交互式配置"
    echo "=============="
    
    # 网关基础信息
    if [[ -z "$GATEWAY_ID" ]]; then
        read -p "网关ID (默认: gateway-$(hostname)): " GATEWAY_ID
        GATEWAY_ID=${GATEWAY_ID:-"gateway-$(hostname)"}
    fi
    
    if [[ -z "$GATEWAY_NAME" ]]; then
        read -p "网关名称 (默认: ${ENVIRONMENT}环境网关): " GATEWAY_NAME
        GATEWAY_NAME=${GATEWAY_NAME:-"${ENVIRONMENT}环境网关"}
    fi
    
    if [[ -z "$GATEWAY_LOCATION" ]]; then
        read -p "网关位置 (默认: 默认位置): " GATEWAY_LOCATION
        GATEWAY_LOCATION=${GATEWAY_LOCATION:-"默认位置"}
    fi
    
    # 功能选择
    echo ""
    echo "功能配置:"
    
    if [[ "$ENVIRONMENT" != "edge" ]]; then
        read -p "启用Web管理界面? (y/N): " ENABLE_WEB
        ENABLE_WEB=${ENABLE_WEB:-"n"}
        
        read -p "启用机器学习引擎? (y/N): " ENABLE_ML
        ENABLE_ML=${ENABLE_ML:-"n"}
        
        read -p "启用数据分析? (y/N): " ENABLE_ANALYTICS
        ENABLE_ANALYTICS=${ENABLE_ANALYTICS:-"n"}
    fi
    
    # 连接配置
    echo ""
    echo "连接配置:"
    
    read -p "MQTT代理地址 (默认: localhost:1883): " MQTT_BROKER
    MQTT_BROKER=${MQTT_BROKER:-"localhost:1883"}
    
    if [[ "$ENVIRONMENT" == "prod" ]]; then
        read -p "InfluxDB地址 (默认: http://influxdb:8086): " INFLUXDB_URL
        INFLUXDB_URL=${INFLUXDB_URL:-"http://influxdb:8086"}
        
        read -p "Redis地址 (默认: redis://redis:6379): " REDIS_URL
        REDIS_URL=${REDIS_URL:-"redis://redis:6379"}
    fi
fi

# 设置默认值
GATEWAY_ID=${GATEWAY_ID:-"gateway-$(hostname)"}
GATEWAY_NAME=${GATEWAY_NAME:-"${ENVIRONMENT}环境网关"}
GATEWAY_LOCATION=${GATEWAY_LOCATION:-"默认位置"}

# 生成主配置文件
echo "📄 生成主配置文件..."

MAIN_CONFIG="$OUTPUT_DIR/gateway.yaml"
TEMPLATE_FILE="$TEMPLATES_DIR/${ENVIRONMENT}-gateway.yml"

if [[ ! -f "$TEMPLATE_FILE" ]]; then
    echo "错误: 找不到模板文件 $TEMPLATE_FILE"
    exit 1
fi

# 检查文件是否存在
if [[ -f "$MAIN_CONFIG" && "$OVERWRITE" != "true" ]]; then
    read -p "配置文件 $MAIN_CONFIG 已存在，是否覆盖? (y/N): " CONFIRM
    if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
        echo "跳过主配置文件生成"
    else
        OVERWRITE=true
    fi
fi

if [[ ! -f "$MAIN_CONFIG" || "$OVERWRITE" == "true" ]]; then
    # 复制模板并替换变量
    cp "$TEMPLATE_FILE" "$MAIN_CONFIG"
    
    # 替换基础变量
    sed -i "s/\${GATEWAY_ID}/$GATEWAY_ID/g" "$MAIN_CONFIG" 2>/dev/null || \
    sed -i '' "s/\${GATEWAY_ID}/$GATEWAY_ID/g" "$MAIN_CONFIG" 2>/dev/null || true
    
    sed -i "s/\${GATEWAY_NAME}/$GATEWAY_NAME/g" "$MAIN_CONFIG" 2>/dev/null || \
    sed -i '' "s/\${GATEWAY_NAME}/$GATEWAY_NAME/g" "$MAIN_CONFIG" 2>/dev/null || true
    
    sed -i "s/\${GATEWAY_LOCATION}/$GATEWAY_LOCATION/g" "$MAIN_CONFIG" 2>/dev/null || \
    sed -i '' "s/\${GATEWAY_LOCATION}/$GATEWAY_LOCATION/g" "$MAIN_CONFIG" 2>/dev/null || true
    
    echo "✅ 生成主配置文件: $MAIN_CONFIG"
fi

# 生成驱动配置文件
if [[ "$ENVIRONMENT" == "prod" ]]; then
    echo "📄 生成驱动配置文件..."
    
    DRIVERS_CONFIG="$OUTPUT_DIR/drivers-config.yml"
    if [[ ! -f "$DRIVERS_CONFIG" || "$OVERWRITE" == "true" ]]; then
        cp "$TEMPLATES_DIR/drivers-config.yml" "$DRIVERS_CONFIG"
        echo "✅ 生成驱动配置文件: $DRIVERS_CONFIG"
    fi
    
    # 生成连接器配置文件
    echo "📄 生成连接器配置文件..."
    
    CONNECTORS_CONFIG="$OUTPUT_DIR/connectors-config.yml"
    if [[ ! -f "$CONNECTORS_CONFIG" || "$OVERWRITE" == "true" ]]; then
        cp "$TEMPLATES_DIR/connectors-config.yml" "$CONNECTORS_CONFIG"
        echo "✅ 生成连接器配置文件: $CONNECTORS_CONFIG"
    fi
    
    # 生成告警配置文件
    echo "📄 生成告警配置文件..."
    
    ALERTS_CONFIG="$OUTPUT_DIR/alerts-config.yml"
    if [[ ! -f "$ALERTS_CONFIG" || "$OVERWRITE" == "true" ]]; then
        cp "$TEMPLATES_DIR/alerts-config.yml" "$ALERTS_CONFIG"
        echo "✅ 生成告警配置文件: $ALERTS_CONFIG"
    fi
fi

# 生成环境变量文件
echo "📄 生成环境变量文件..."

ENV_FILE="$OUTPUT_DIR/.env"
if [[ ! -f "$ENV_FILE" || "$OVERWRITE" == "true" ]]; then
    cat > "$ENV_FILE" << EOF
# Edge Gateway 环境变量配置
# 生成时间: $(date)
# 环境类型: $ENVIRONMENT

# 网关基础信息
GATEWAY_ID=$GATEWAY_ID
GATEWAY_NAME="$GATEWAY_NAME"
GATEWAY_LOCATION="$GATEWAY_LOCATION"

# 服务配置
RUST_LOG=info
GATEWAY_CONFIG=./config/gateway.yaml

# 安全配置 (生产环境请修改这些密钥)
JWT_SECRET="change-me-32-byte-jwt-secret-key"
ENCRYPTION_KEY="change-me-32-byte-encryption-key"

EOF

    if [[ "$ENVIRONMENT" != "edge" ]]; then
        cat >> "$ENV_FILE" << EOF
# MQTT配置
MQTT_BROKER=${MQTT_BROKER:-localhost:1883}
MQTT_USERNAME=gateway
MQTT_PASSWORD=change-me-mqtt-password

EOF
    fi

    if [[ "$ENVIRONMENT" == "prod" ]]; then
        cat >> "$ENV_FILE" << EOF
# 数据库配置
INFLUXDB_URL=${INFLUXDB_URL:-http://influxdb:8086}
INFLUXDB_DATABASE=gateway_data
INFLUXDB_USERNAME=gateway
INFLUXDB_PASSWORD=change-me-influxdb-password

# Redis配置
REDIS_URL=${REDIS_URL:-redis://redis:6379}
REDIS_PASSWORD=change-me-redis-password

# 告警配置
ENABLE_EMAIL_ALERTS=false
SMTP_SERVER=smtp.company.com
SMTP_USERNAME=gateway@company.com
SMTP_PASSWORD=change-me-smtp-password
ALERT_EMAIL_TO=admin@company.com

# 安全配置
ENABLE_TLS=false
ENABLE_RBAC=true

EOF
    fi

    chmod 600 "$ENV_FILE"
    echo "✅ 生成环境变量文件: $ENV_FILE"
    echo "⚠️  注意: 请修改 $ENV_FILE 中的默认密码!"
fi

# 生成启动脚本
echo "📄 生成启动脚本..."

START_SCRIPT="$OUTPUT_DIR/start-gateway.sh"
if [[ ! -f "$START_SCRIPT" || "$OVERWRITE" == "true" ]]; then
    cat > "$START_SCRIPT" << 'EOF'
#!/bin/bash
# Edge Gateway 启动脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 加载环境变量
if [[ -f ".env" ]]; then
    export $(grep -v '^#' .env | xargs)
fi

# 验证配置文件
if [[ ! -f "gateway.yaml" ]]; then
    echo "错误: 找不到配置文件 gateway.yaml"
    exit 1
fi

# 创建必要目录
mkdir -p data logs temp models

# 启动网关
echo "🚀 启动 Edge Gateway..."
echo "📁 配置文件: $PWD/gateway.yaml"
echo "🌍 环境: ${ENVIRONMENT:-未知}"

exec edge-gateway --config gateway.yaml
EOF

    chmod +x "$START_SCRIPT"
    echo "✅ 生成启动脚本: $START_SCRIPT"
fi

# 生成停止脚本
STOP_SCRIPT="$OUTPUT_DIR/stop-gateway.sh"
if [[ ! -f "$STOP_SCRIPT" || "$OVERWRITE" == "true" ]]; then
    cat > "$STOP_SCRIPT" << 'EOF'
#!/bin/bash
# Edge Gateway 停止脚本

echo "🛑 停止 Edge Gateway..."

# 查找并终止进程
PIDS=$(pgrep -f "edge-gateway" || true)
if [[ -n "$PIDS" ]]; then
    echo "发现进程: $PIDS"
    kill -TERM $PIDS
    
    # 等待进程优雅关闭
    sleep 5
    
    # 强制终止未关闭的进程
    PIDS=$(pgrep -f "edge-gateway" || true)
    if [[ -n "$PIDS" ]]; then
        echo "强制终止进程: $PIDS"
        kill -KILL $PIDS
    fi
    
    echo "✅ Edge Gateway 已停止"
else
    echo "ℹ️  Edge Gateway 未运行"
fi
EOF

    chmod +x "$STOP_SCRIPT"
    echo "✅ 生成停止脚本: $STOP_SCRIPT"
fi

# 生成README文件
echo "📄 生成说明文档..."

README_FILE="$OUTPUT_DIR/README.md"
if [[ ! -f "$README_FILE" || "$OVERWRITE" == "true" ]]; then
    cat > "$README_FILE" << EOF
# Edge Gateway 配置文件

本目录包含为 **$ENVIRONMENT** 环境生成的配置文件。

## 生成信息

- **生成时间**: $(date)
- **环境类型**: $ENVIRONMENT
- **网关ID**: $GATEWAY_ID
- **网关名称**: $GATEWAY_NAME
- **网关位置**: $GATEWAY_LOCATION

## 文件说明

### 核心文件
- `gateway.yaml` - 主配置文件
- \`.env\` - 环境变量文件
- \`start-gateway.sh\` - 启动脚本
- \`stop-gateway.sh\` - 停止脚本

EOF

    if [[ "$ENVIRONMENT" == "prod" ]]; then
        cat >> "$README_FILE" << EOF
### 生产环境配置文件
- \`drivers-config.yml\` - 驱动配置
- \`connectors-config.yml\` - 连接器配置
- \`alerts-config.yml\` - 告警配置

EOF
    fi

    cat >> "$README_FILE" << EOF
## 使用方法

### 1. 配置检查
在启动前，请检查并修改以下配置：

1. **环境变量** (\`.env\`):
   - 修改默认密码和密钥
   - 配置外部服务地址
   
2. **主配置文件** (`gateway.yaml`):
   - 检查驱动配置
   - 验证连接器设置
   - 调整系统参数

### 2. 启动服务
\`\`\`bash
# 直接启动
./start-gateway.sh

# 或者手动启动
edge-gateway --config gateway.yaml
\`\`\`

### 3. 验证运行
\`\`\`bash
# 检查健康状态
curl http://localhost:8080/healthz

# 查看指标
curl http://localhost:9090/metrics
\`\`\`

### 4. 停止服务
\`\`\`bash
./stop-gateway.sh
\`\`\`

## 服务端点

- **Web管理界面**: http://localhost:8090
- **REST API**: http://localhost:8080
- **Prometheus指标**: http://localhost:9090

## 注意事项

⚠️ **安全提醒**:
- 生产环境必须修改默认密码
- 建议启用TLS/SSL加密
- 定期更新认证密钥

📝 **配置提醒**:
- 根据实际环境调整资源限制
- 配置适当的告警规则
- 定期备份配置文件

## 故障排除

### 常见问题
1. **服务启动失败**
   - 检查配置文件语法: \`edge-gateway --config gateway.yml --validate\`
   - 查看日志文件: \`tail -f logs/gateway.log\`

2. **连接失败**
   - 检查网络连通性
   - 验证认证信息
   - 查看防火墙设置

3. **性能问题**
   - 监控系统资源使用
   - 调整线程池大小
   - 优化批量处理参数

### 获取帮助
- 查看日志: \`logs/gateway.log\`
- 系统状态: \`curl http://localhost:8090/health\`
- 配置验证: \`edge-gateway --validate-config\`

EOF

    echo "✅ 生成说明文档: $README_FILE"
fi

echo ""
echo "🎉 配置生成完成!"
echo ""
echo "📋 生成的文件:"
echo "  主配置: $MAIN_CONFIG"
echo "  环境变量: $ENV_FILE"
echo "  启动脚本: $START_SCRIPT"
echo "  停止脚本: $STOP_SCRIPT"
echo "  说明文档: $README_FILE"

if [[ "$ENVIRONMENT" == "prod" ]]; then
    echo "  驱动配置: $OUTPUT_DIR/drivers-config.yml"
    echo "  连接器配置: $OUTPUT_DIR/connectors-config.yml"
    echo "  告警配置: $OUTPUT_DIR/alerts-config.yml"
fi

echo ""
echo "💡 下一步:"
echo "  1. 检查并修改 $ENV_FILE 中的配置"
echo "  2. 根据需要调整 $MAIN_CONFIG"
echo "  3. 运行 $START_SCRIPT 启动服务"
echo ""
echo "📖 详细信息请查看: $README_FILE"