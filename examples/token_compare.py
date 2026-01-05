#!/usr/bin/env python3
import json
import subprocess
import tiktoken

enc = tiktoken.get_encoding("cl100k_base")

def count_tokens(text: str) -> int:
    return len(enc.encode(text))

def generate_medium_json(num_users: int = 50) -> dict:
    return {
        "users": [
            {
                "id": i,
                "name": f"User_{i}",
                "email": f"user{i}@example.com",
                "age": 20 + (i % 50),
                "active": i % 2 == 0,
                "tags": ["developer", "python", "rust"] if i % 3 == 0 else ["user"],
                "metadata": {
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-12-01T00:00:00Z",
                    "login_count": i * 10
                }
            }
            for i in range(1, num_users + 1)
        ],
        "pagination": {
            "page": 1,
            "per_page": 50,
            "total": num_users,
            "total_pages": (num_users + 49) // 50
        },
        "meta": {
            "version": "1.0.0",
            "api_version": "v2",
            "request_id": "abc123def456"
        }
    }

def format_with_jf(json_str: str, mode: str, args: list = None) -> str:
    cmd = ["./target/release/jf", "format", "--mode", mode]
    if args:
        cmd.extend(args)
        
    result = subprocess.run(
        cmd,
        input=json_str,
        capture_output=True,
        text=True
    )
    return result.stdout.strip()

def print_comparison(title: str, baseline: str, baseline_name: str, jf_output: str, jf_name: str):
    baseline_tokens = count_tokens(baseline)
    jf_tokens = count_tokens(jf_output)
    savings = (1 - jf_tokens / baseline_tokens) * 100
    
    print(f"\n{'='*60}")
    print(f" {title}")
    print(f"{'='*60}")
    print(f"{baseline_name} 大小: {len(baseline):,} bytes")
    print(f"{jf_name} 大小: {len(jf_output):,} bytes")
    print(f"")
    print(f"{baseline_name} Tokens: {baseline_tokens:,}")
    print(f"{jf_name} Tokens: {jf_tokens:,}")
    print(f"")
    print(f"Token 节省: {savings:.1f}%")
    print(f"{'='*60}")
    
    print(f"\n--- {jf_name} 输出预览 (前 500 字符) ---")
    preview = jf_output[:500]
    if len(jf_output) > 500:
        preview += "..."
    print(preview)

def main():
    print("=" * 60)
    print(" Smart Mode vs Pretty Mode Token 对比")
    print(" 使用 tiktoken (cl100k_base) 计算")
    print("=" * 60)
    
    print("\n\n" + "=" * 60)
    print(" 测试: 中型 JSON (50 用户)")
    print("=" * 60)
    
    medium_data = generate_medium_json(50)
    # 标准 Pretty 格式作为基准
    baseline = json.dumps(medium_data, ensure_ascii=False, indent=2)
    
    # Smart 模式 (默认 limit 80)
    smart_80 = format_with_jf(json.dumps(medium_data), "smart", ["--inline-limit", "80"])
    
    # Smart 模式 (更激进 limit 120)
    smart_120 = format_with_jf(json.dumps(medium_data), "smart", ["--inline-limit", "120"])
    
    # Compact 模式 (极限)
    compact = format_with_jf(json.dumps(medium_data), "compact")
    
    print_comparison("Smart (Limit 80) vs Pretty", baseline, "Pretty", smart_80, "Smart(80)")
    print_comparison("Smart (Limit 120) vs Pretty", baseline, "Pretty", smart_120, "Smart(120)")
    print_comparison("Compact vs Pretty", baseline, "Pretty", compact, "Compact")
    
    baseline_tokens = count_tokens(baseline)
    smart_80_tokens = count_tokens(smart_80)
    smart_120_tokens = count_tokens(smart_120)
    compact_tokens = count_tokens(compact)
    
    print(f"""
┌─────────────────────┬───────────────┬───────────────┬────────────┐
│ 模式                │ Tokens        │ 相对 Pretty    │ 可读性      │
├─────────────────────┼───────────────┼───────────────┼────────────┤
│ Pretty (基准)       │ {baseline_tokens:>13,} │ 100.0%        │ ⭐⭐⭐⭐⭐      │
│ Smart (Limit 80)    │ {smart_80_tokens:>13,} │ {smart_80_tokens/baseline_tokens*100:>5.1f}%        │ ⭐⭐⭐⭐       │
│ Smart (Limit 120)   │ {smart_120_tokens:>13,} │ {smart_120_tokens/baseline_tokens*100:>5.1f}%        │ ⭐⭐⭐        │
│ Compact             │ {compact_tokens:>13,} │ {compact_tokens/baseline_tokens*100:>5.1f}%        │ ⭐           │
└─────────────────────┴───────────────┴───────────────┴────────────┘
""")

if __name__ == "__main__":
    main()
