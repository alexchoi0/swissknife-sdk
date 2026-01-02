use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::net::{Ipv4Addr, Ipv6Addr};

fn bench_ssrf_ipv4_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("ssrf_ipv4");

    let addrs = vec![
        ("public", Ipv4Addr::new(8, 8, 8, 8)),
        ("loopback", Ipv4Addr::new(127, 0, 0, 1)),
        ("private", Ipv4Addr::new(10, 0, 0, 1)),
        ("metadata", Ipv4Addr::new(169, 254, 169, 254)),
    ];

    for (name, ip) in addrs {
        group.bench_with_input(BenchmarkId::new("is_restricted", name), &ip, |b, ip| {
            b.iter(|| secretary::security::ssrf::is_restricted_ipv4(black_box(*ip)))
        });
    }
    group.finish();
}

fn bench_ssrf_ipv6_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("ssrf_ipv6");

    let addrs = vec![
        ("loopback", Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
        ("ula", Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1)),
        (
            "link_local",
            Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1),
        ),
        (
            "mapped_loopback",
            Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 0x0001),
        ),
    ];

    for (name, ip) in addrs {
        group.bench_with_input(BenchmarkId::new("is_restricted", name), &ip, |b, ip| {
            b.iter(|| secretary::security::ssrf::is_restricted_ipv6(black_box(*ip)))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ssrf_ipv4_check, bench_ssrf_ipv6_check);
criterion_main!(benches);
