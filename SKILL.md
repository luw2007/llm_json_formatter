---
name: llm-json-formatter
description: Format, analyze, and manipulate JSON data with LLM-optimized output that balances readability and token efficiency. Use when working with JSON data that needs to be formatted (smart/compact/pretty modes), analyzed (structure metadata), searched (JSON path queries), or when you need to list all available paths in a JSON object. Particularly useful for preparing JSON for LLM consumption with 40% token reduction while maintaining readability.
---

# LLM JSON Formatter

优化的 JSON 格式化工具，专为 LLM 使用而设计，平衡可读性和 token 效率。

## 核心功能

通过 `jf` 命令行工具提供以下功能：

1. **格式化 JSON** - 使用 Smart/Compact/Pretty 三种模式
2. **分析结构** - 获取深度、大小、键数量等元数据
3. **路径查询** - 使用 JSON 路径查询特定值
4. **列出路径** - 列出 JSON 对象中所有可用路径

## 使用方法

### 格式化 JSON

```bash
jf format [OPTIONS]
```

**常用选项：**

- `--mode <MODE>`: 格式化模式（smart/compact/pretty），默认 smart
- `--sort <STRATEGY>`: 键排序策略（alphabetic/smart），默认 alphabetic
- `--indent <NUM>`: 缩进空格数，默认 2
- `--entities <PATHS>`: 手动指定实体路径，如 "users[*],orders[*]"
- `--entity-threshold <NUM>`: 自动检测实体的阈值，默认 2000

**示例：**

```bash
# 使用 Smart 模式（默认）- 自动检测实体并保持单行
echo '{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}' | jf format

# 紧凑模式 - 最小 token 使用
echo '{"data":"value"}' | jf format --mode compact

# Pretty 模式 - 最大可读性
echo '{"data":"value"}' | jf format --mode pretty

# 手动指定实体
echo '{"users":[...]}' | jf format --entities "users[*]"

# 智能键排序（id/name 优先）
echo '{"_internal":1,"id":100,"name":"test"}' | jf format --sort smart
```

### 分析 JSON 结构

```bash
jf analyze
```

获取 JSON 的结构信息：字节大小、最大深度、对象数量、键总数、数组数量、最大数组长度。

**示例：**

```bash
echo '{"users":[{"id":1,"name":"Alice"}]}' | jf analyze
# 输出：
# JSON Analysis:
#   Byte Size: 35 bytes
#   Max Depth: 3
#   Object Count: 2
#   Total Keys: 2
#   Array Count: 1
#   Max Array Length: 1
```

### 路径查询

```bash
jf search --path <PATH>
```

使用 JSON 路径查询特定值（如 `users[0].name`）。

**示例：**

```bash
echo '{"users":[{"id":1,"name":"Alice"}]}' | jf search --path "users[0].name"
# 输出：Alice
```

### 列出所有路径

```bash
jf paths
```

列出 JSON 对象中所有可用的路径。

**示例：**

```bash
echo '{"users":[{"id":1,"name":"Alice"}]}' | jf paths
# 输出：
# users
# users[0]
# users[0].id
# users[0].name
```

## 格式化模式详解

### Smart 模式（推荐）

**特点：** 自动检测"实体对象"（如用户记录、订单项）并保持单行，同时展开整体结构。

**Token 节省：** 相比 Pretty 模式节省约 40%

**何时使用：** 准备 JSON 数据给 LLM 消费时的默认选择

**示例：**

```json
{
  "users": [
    {"id":1,"name":"Alice","email":"alice@example.com"},
    {"id":2,"name":"Bob","email":"bob@example.com"}
  ]
}
```

### Compact 模式

**特点：** 完全压缩，单行输出，无空格

**Token 节省：** 最大化 token 效率

**何时使用：** 当 token 限制严格且 LLM 能处理压缩格式时

**示例：**

```json
{"age":30,"city":"NYC","name":"Alice"}
```

### Pretty 模式

**特点：** 标准缩进输出，完全展开

**可读性：** 最大化可读性

**何时使用：** 人类阅读或调试时

**示例：**

```json
{
  "age": 30,
  "city": "NYC",
  "name": "Alice"
}
```

## 实体检测

### 自动检测

基于 P90 长度分析自动识别实体。如果数组项的 P90 长度 ≤ `entity-threshold`（默认 2000），则标记为实体。

```bash
jf format --entity-threshold 1500
```

### 手动指定

使用 `--entities` 选项手动指定实体路径：

```bash
# 逗号分隔格式
jf format --entities "users[*],orders[*],products[*]"

# JSON 数组格式
jf format --entities '["users[*]","orders[*]"]'

# 禁用自动检测
jf format --entity-threshold 0 --entities "users[*]"
```

## 键排序策略

### Alphabetic（默认）

按字母顺序排序键：

```json
{"age":30,"city":"NYC","name":"Alice"}
```

### Smart

按重要性权重排序（id/name/type 优先，_internal 靠后）：

```json
{"id":100,"name":"Alice","city":"NYC","_internal":2}
```

使用方式：

```bash
jf format --sort smart
```

## 输入/输出

所有命令默认从 stdin 读取，输出到 stdout：

```bash
# 从 stdin
echo '{"data":"value"}' | jf format

# 从文件
cat data.json | jf format

# 保存到文件
jf format < input.json > output.json
```

## 工作流示例

### 场景 1：准备 JSON 给 LLM

```bash
# 使用 Smart 模式优化 token 使用
curl https://api.example.com/data | jf format --mode smart
```

### 场景 2：查找特定值

```bash
# 先列出所有路径
cat data.json | jf paths

# 然后查询特定路径
cat data.json | jf search --path "config.database.host"
```

### 场景 3：分析大型 JSON

```bash
# 先分析结构
cat large.json | jf analyze

# 如果数组很多，手动指定实体以优化输出
cat large.json | jf format --entities "users[*],orders[*]"
```

### 场景 4：调试和对比

```bash
# Pretty 模式用于人类阅读
cat data.json | jf format --mode pretty > readable.json

# Smart 模式用于 LLM
cat data.json | jf format --mode smart > llm-optimized.json

# 对比文件大小
ls -lh readable.json llm-optimized.json
```

## 注意事项

1. **jf 二进制位置：** 确保 `jf` 在 PATH 中或使用完整路径
2. **大型 JSON：** Smart 模式对大型 JSON（>10MB）可能需要较长处理时间
3. **实体阈值：** 调整 `--entity-threshold` 可以控制哪些数组项被视为实体
4. **键排序：** Smart 排序对于有明确 id/name 字段的数据最有效

## Token 效率对比

以包含 100 条用户记录的 JSON 为例：

| 模式 | 约估 Token 数 | 相比 Pretty 节省 |
|------|--------------|-----------------|
| Pretty | 1000 | - |
| Smart | 600 | 40% |
| Compact | 500 | 50% |

## 错误处理

如果命令失败，检查：

1. JSON 格式是否有效
2. 路径查询语法是否正确（如 `users[0].name`）
3. `jf` 二进制是否可访问
4. 输入是否为空

常见错误：

```bash
# 错误：无效的 JSON
echo 'not json' | jf format
# Error: JSON parsing failed

# 错误：路径不存在
echo '{"a":1}' | jf search --path "b.c"
# Path not found: b.c
```
