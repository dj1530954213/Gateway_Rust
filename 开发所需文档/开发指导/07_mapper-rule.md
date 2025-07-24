# 07 – Mapper / Rule‑Engine 设计文档 (v1‑draft)

## 1. 作用
* 将 `DataFrame(tag,value)` **重命名 / 换算 / 聚合 / 过滤**  
* 生成告警 Frame (`Kind::Data`, tag=`sys.alarm.*`)  
* 可热加载；不阻塞采集路径

## 2. 公共接口
```rust
pub trait Mapper: Send + Sync {
    /// 每次收到 DataFrame 调用
    fn on_data(&self, df: &DataFrame) -> Vec<DataFrame>;
}
```
系统仅提供 **InternalMapper**，用户可后期动态加载 `DynMapper(.so)`。

## 3. DSL
### 3.1 YAML mapping
```yaml
- match: plant.press_bar_raw
  rename: plant.press_bar
  expr: "value / 100.0"
- match: plant.flow_m3h_raw
  rename: plant.flow_m3h
  expr: "value / 10.0"
```
### 3.2 Lua rule (单独文件)
```lua
if tag == "plant.temp_c" and value > 80 then
  emit("sys.alarm.temp_hi", 1)
end
```

## 4. 数据流
Driver → FrameBus → **Mapper subscribe(Filter::All)**  
Mapper emit → Bus.publish(newDF)  
*环内 0‑copy：payload bytes 直接转发*

## 5. 配置 & 热更新
* mapping.yml vSN   
* rule.lua 支持 `SIGHUP` 热重载  
* Diff 逻辑：仅影响 Mapper，不 Pause 采集

## 6. 执行模型
* YAML 映射：预编译为 `Vec<Regex,FnPtr>`  
* Lua：`mlua`，每条 DF 复用同 VM；20 µs 限时

## 7. 指标
| metric | 标签 | 含义 |
| mapper_exec_ns_bucket | mapper_id | 执行时长直方图 |
| mapper_emit_total | mapper_id | 生成帧计数 |

## 8. 测试
* rename_scale_ok / lua_alarm / hot_reload_yaml / exec_timeout

## 9. 性能
10 k fps, 平均 delay ≤ 30 µs

## 10. 里程碑
0.1 YAML rename/scale；0.2 Lua single file；1.0 多文件 + WASM