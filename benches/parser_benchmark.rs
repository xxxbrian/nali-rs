use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fake::faker::company::raw::*;
use fake::faker::internet::raw::*;
use fake::faker::lorem::raw::*;
use fake::faker::name::raw::*;
use fake::locales::EN;
use fake::Fake;
use nali::{geo::fakegeo::FakeGeo, FastParser, Parser, RegexParser};
use rand::{thread_rng, Rng};

// Generate random IPv4 address
fn gen_ipv4() -> String {
    IPv4(EN).fake()
}

// Generate random IPv6 address
fn gen_ipv6() -> String {
    IPv6(EN).fake()
}

// Generate random text without IPs
fn gen_random_text() -> String {
    let mut rng = thread_rng();
    let choices: Vec<String> = vec![
        Word(EN).fake(),
        Name(EN).fake(),
        CompanyName(EN).fake(),
        Username(EN).fake(),
        DomainSuffix(EN).fake(),
    ];
    choices[rng.gen_range(0..choices.len())].clone()
}

// Join elements with random separator
fn join_with_random_sep(elements: Vec<String>) -> String {
    let mut rng = thread_rng();
    let result = elements
        .into_iter()
        .map(|elem| {
            let sep_rand: f64 = rng.gen();
            let separator = if sep_rand < 0.7 {
                ""
            } else if sep_rand < 0.8 {
                " "
            } else {
                match rng.gen_range(0..2) {
                    0 => ":",
                    _ => ".",
                }
            };
            format!("{}{}", elem, separator)
        })
        .collect::<String>();
    result
}

// Generate test data based on percentages
fn generate_test_data(
    noise_pct: f64,
    ipv4_pct: f64,
    ipv6_pct: f64,
    total_elements: usize,
) -> String {
    let mut rng = thread_rng();
    let mut elements = Vec::new();

    let noise_count = (total_elements as f64 * noise_pct) as usize;
    let ipv4_count = (total_elements as f64 * ipv4_pct) as usize;
    let ipv6_count = (total_elements as f64 * ipv6_pct) as usize;

    // Add random text
    for _ in 0..noise_count {
        elements.push(gen_random_text());
    }

    // Add IPv4 addresses
    for _ in 0..ipv4_count {
        elements.push(gen_ipv4());
    }

    // Add IPv6 addresses
    for _ in 0..ipv6_count {
        elements.push(gen_ipv6());
    }

    // Shuffle elements
    for i in (1..elements.len()).rev() {
        let j = rng.gen_range(0..=i);
        elements.swap(i, j);
    }

    join_with_random_sep(elements)
}

fn bench_scenario(
    c: &mut Criterion,
    name: &str,
    noise_pct: f64,
    ipv4_pct: f64,
    ipv6_pct: f64,
    size: usize,
) {
    let geo = FakeGeo::new();
    let fast_parser = FastParser::default();
    let regex_parser = RegexParser::default();

    let test_data = generate_test_data(noise_pct, ipv4_pct, ipv6_pct, size);

    let group_name = format!("{} (size: {})", name, size);
    let mut group = c.benchmark_group(&group_name);

    group.bench_function("FastParser", |b| {
        b.iter(|| {
            black_box(fast_parser.parse(&test_data, &geo));
        });
    });

    group.bench_function("RegexParser", |b| {
        b.iter(|| {
            black_box(regex_parser.parse(&test_data, &geo));
        });
    });

    group.finish();
}

fn run_size_scenarios(c: &mut Criterion, noise_pct: f64, ipv4_pct: f64, ipv6_pct: f64, name: &str) {
    let sizes = [50, 300, 1000, 5000, 10000, 100000];
    for &size in sizes.iter() {
        bench_scenario(c, name, noise_pct, ipv4_pct, ipv6_pct, size);
    }
}

fn parser_benchmark(c: &mut Criterion) {
    // Pure IP scenarios
    run_size_scenarios(c, 0.0, 1.0, 0.0, "100% IPv4");
    run_size_scenarios(c, 0.0, 0.0, 1.0, "100% IPv6");
    run_size_scenarios(c, 0.0, 0.5, 0.5, "50-50 IPv4-IPv6");

    // 30% noise scenarios
    run_size_scenarios(c, 0.3, 0.7, 0.0, "30% noise, 70% IPv4");
    run_size_scenarios(c, 0.3, 0.0, 0.7, "30% noise, 70% IPv6");
    run_size_scenarios(c, 0.3, 0.35, 0.35, "30% noise, 35-35 IPv4-IPv6");

    // 60% noise scenarios
    run_size_scenarios(c, 0.6, 0.4, 0.0, "60% noise, 40% IPv4");
    run_size_scenarios(c, 0.6, 0.0, 0.4, "60% noise, 40% IPv6");
    run_size_scenarios(c, 0.6, 0.2, 0.2, "60% noise, 20-20 IPv4-IPv6");

    // 90% noise scenarios
    run_size_scenarios(c, 0.9, 0.1, 0.0, "90% noise, 10% IPv4");
    run_size_scenarios(c, 0.9, 0.0, 0.1, "90% noise, 10% IPv6");
    run_size_scenarios(c, 0.9, 0.05, 0.05, "90% noise, 5-5 IPv4-IPv6");

    // Pure noise scenario
    run_size_scenarios(c, 1.0, 0.0, 0.0, "100% noise");
}

criterion_group!(benches, parser_benchmark);
criterion_main!(benches);
