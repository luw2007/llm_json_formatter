use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llm_json_formatter::{Config, FormatMode, LlmJsonFormatter, SortStrategy};
use rand::Rng;
use serde_json::{json, Value};

fn generate_small_json() -> String {
    json!({
        "id": 12345,
        "name": "Alice Johnson",
        "email": "alice@example.com",
        "age": 28,
        "active": true,
        "score": 95.5
    })
    .to_string()
}

fn generate_medium_json(array_size: usize) -> String {
    let mut rng = rand::thread_rng();
    let users: Vec<Value> = (0..array_size)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("User_{}", i),
                "email": format!("user{}@example.com", i),
                "age": rng.gen_range(18..65),
                "score": rng.gen_range(0.0..100.0),
                "active": rng.gen_bool(0.7),
                "tags": ["tag1", "tag2", "tag3"],
                "metadata": {
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-02T00:00:00Z",
                    "version": 1
                }
            })
        })
        .collect();

    json!({
        "total": array_size,
        "page": 1,
        "per_page": array_size,
        "users": users,
        "meta": {
            "api_version": "v2",
            "request_id": "abc123"
        }
    })
    .to_string()
}

fn generate_large_json(array_size: usize, nested_depth: usize) -> String {
    let mut rng = rand::thread_rng();

    fn create_nested(depth: usize, max_depth: usize, rng: &mut impl Rng) -> Value {
        if depth >= max_depth {
            return json!({
                "leaf_id": rng.gen::<u32>(),
                "leaf_value": format!("value_{}", rng.gen::<u16>()),
                "leaf_score": rng.gen_range(0.0..100.0)
            });
        }
        json!({
            "level": depth,
            "node_id": rng.gen::<u32>(),
            "children": [
                create_nested(depth + 1, max_depth, rng),
                create_nested(depth + 1, max_depth, rng)
            ],
            "metadata": {
                "created": "2024-01-01",
                "tags": ["a", "b", "c"]
            }
        })
    }

    let items: Vec<Value> = (0..array_size)
        .map(|i| {
            json!({
                "item_id": i,
                "item_name": format!("Item_{}", i),
                "nested_data": create_nested(0, nested_depth, &mut rng),
                "attributes": {
                    "color": "red",
                    "size": "large",
                    "weight": rng.gen_range(1.0..100.0),
                    "dimensions": {
                        "width": rng.gen_range(10..100),
                        "height": rng.gen_range(10..100),
                        "depth": rng.gen_range(10..100)
                    }
                },
                "history": (0..5).map(|j| json!({
                    "event_id": j,
                    "timestamp": format!("2024-01-0{}T00:00:00Z", j + 1),
                    "action": "update"
                })).collect::<Vec<_>>()
            })
        })
        .collect();

    json!({
        "dataset": {
            "name": "benchmark_dataset",
            "version": "1.0.0",
            "items": items,
            "statistics": {
                "total_items": array_size,
                "avg_score": 50.0,
                "max_depth": nested_depth
            }
        },
        "config": {
            "compression": true,
            "encryption": false,
            "format": "json"
        }
    })
    .to_string()
}

fn bench_format_small(c: &mut Criterion) {
    let json = generate_small_json();
    let byte_size = json.len();

    let mut group = c.benchmark_group("format_small");
    group.throughput(Throughput::Bytes(byte_size as u64));

    group.bench_function("alphabetic", |b| {
        b.iter(|| {
            let mut formatter = LlmJsonFormatter::new(Config {
                sort_strategy: SortStrategy::Alphabetic,
                ..Default::default()
            });
            formatter.format(black_box(&json)).unwrap()
        })
    });

    group.bench_function("smart", |b| {
        b.iter(|| {
            let mut formatter = LlmJsonFormatter::new(Config {
                sort_strategy: SortStrategy::Smart,
                ..Default::default()
            });
            formatter.format(black_box(&json)).unwrap()
        })
    });

    group.finish();
}

fn bench_format_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_medium");

    for size in [100, 500, 1000] {
        let json = generate_medium_json(size);
        let byte_size = json.len();

        group.throughput(Throughput::Bytes(byte_size as u64));

        group.bench_with_input(BenchmarkId::new("smart", size), &json, |b, json| {
            b.iter(|| {
                let mut formatter = LlmJsonFormatter::new(Config {
                    mode: FormatMode::Smart,
                    ..Default::default()
                });
                formatter.format(black_box(json)).unwrap()
            })
        });

        group.bench_with_input(BenchmarkId::new("compact", size), &json, |b, json| {
            b.iter(|| {
                let mut formatter = LlmJsonFormatter::new(Config {
                    mode: FormatMode::Compact,
                    ..Default::default()
                });
                formatter.format(black_box(json)).unwrap()
            })
        });

        group.bench_with_input(BenchmarkId::new("pretty", size), &json, |b, json| {
            b.iter(|| {
                let mut formatter = LlmJsonFormatter::new(Config {
                    mode: FormatMode::Pretty,
                    ..Default::default()
                });
                formatter.format(black_box(json)).unwrap()
            })
        });
    }

    group.finish();
}

fn bench_format_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_large");

    for (array_size, depth) in [(100, 4), (500, 5), (1000, 6)] {
        let json = generate_large_json(array_size, depth);
        let byte_size = json.len();
        let label = format!("{}x{}", array_size, depth);

        group.throughput(Throughput::Bytes(byte_size as u64));

        group.bench_with_input(BenchmarkId::new("smart", &label), &json, |b, json| {
            b.iter(|| {
                let mut formatter = LlmJsonFormatter::new(Config {
                    mode: FormatMode::Smart,
                    ..Default::default()
                });
                formatter.format(black_box(json)).unwrap()
            })
        });

        group.bench_with_input(BenchmarkId::new("compact", &label), &json, |b, json| {
            b.iter(|| {
                let mut formatter = LlmJsonFormatter::new(Config {
                    mode: FormatMode::Compact,
                    ..Default::default()
                });
                formatter.format(black_box(json)).unwrap()
            })
        });

        group.bench_with_input(BenchmarkId::new("pretty", &label), &json, |b, json| {
            b.iter(|| {
                let mut formatter = LlmJsonFormatter::new(Config {
                    mode: FormatMode::Pretty,
                    ..Default::default()
                });
                formatter.format(black_box(json)).unwrap()
            })
        });
    }

    group.finish();
}

fn bench_json_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_index");

    for size in [100, 500, 1000] {
        let json = generate_medium_json(size);
        let byte_size = json.len();

        group.throughput(Throughput::Bytes(byte_size as u64));

        group.bench_with_input(BenchmarkId::new("build", size), &json, |b, json| {
            b.iter(|| llm_json_formatter::JsonIndex::build(black_box(json)).unwrap())
        });

        let index = llm_json_formatter::JsonIndex::build(&json).unwrap();
        group.bench_with_input(BenchmarkId::new("search", size), &index, |b, index| {
            b.iter(|| index.search(black_box("users[0].name")))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_format_small,
    bench_format_medium,
    bench_format_large,
    bench_json_index,
);

criterion_main!(benches);
