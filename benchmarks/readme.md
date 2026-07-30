# Benchmarks

Each subfolder is an independent Cargo package that compares `basenc` with one
specific crate. They intentionally do not share a virtual workspace, lockfile,
or `target` directory.

Run a suite from the repository root:

```console
cargo +nightly bench --manifest-path benchmarks/base64/Cargo.toml
cargo +nightly bench --manifest-path benchmarks/base64_turbo/Cargo.toml
cargo +nightly bench --manifest-path benchmarks/simple_base64/Cargo.toml
```

## Results

Throughput in MB/s (higher is better), measured my machine with `rustc 1.98.0-nightly (6bdf43094 2026-06-01)`.
All output buffers are allocated before the timed loop and reused.
The relative column expresses `basenc` throughput as a multiple of the other crate, so higher is always better.

Results are from one run and will vary with the machine and toolchain.

### `base64` 0.22.1

| Operation | `basenc` | `base64` | Relative |
|---|---:|---:|---:|
| encode | 14,899 | 2,675 | × 5.57 |
| decode | 26,206 | 3,524 | × 7.44 |

### `base64-turbo` 0.2.0

| Operation | `basenc` | `base64-turbo` | Relative |
|---|---:|---:|---:|
| encode | 15,168 | 21,045 | × 0.72 |
| decode | 25,808 | 39,857 | × 0.65 |

### `simple-base64` 0.23.2

| Operation | `basenc` | `simple-base64` | Relative |
|---|---:|---:|---:|
| encode | 14,988 | 2,795 | × 5.36 |
| decode | 26,104 | 3,670 | × 7.11 |
