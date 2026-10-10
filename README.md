# Sewlore Sewing Math for Rust

A dependency-free Rust library and offline JSON command line tool for comparing sewing measurement readings. It computes stretch, signed residual change, recovered extension, length conversion, and independent X/Y print-scale errors. The typed `Length` API associates every reading with `mm`, `cm`, or `in` and rejects negative, NaN, and infinite values.

Package guide, formulas, and examples: [Rust sewing measurement arithmetic](https://sewlore.com/pages/rust-sewing-measurements). Related reading: [Sewlore fabric measurement methods](https://sewlore.com/pages/fabric-measurement-methods-guide).

## Library

```toml
[dependencies]
sewlore-sewing-math = "0.1"
```

```rust
use sewlore_sewing_math::{stretch, Length, Unit};

fn main() -> Result<(), sewlore_sewing_math::Error> {
    let r = stretch(
        Length::new(10.0, Unit::Centimetre)?,
        Length::new(140.0, Unit::Millimetre)?,
        Length::new(10.5, Unit::Centimetre)?,
    )?;
    assert_eq!(r.stretch_percent, 40.0);
    assert_eq!(r.residual_change_percent, 5.0);
    assert_eq!(r.recovered_extension_percent, Some(87.5));
    Ok(())
}
```

These are hypothetical arithmetic examples, not physical fabric test data.

## CLI

Install the binary from the published package with `cargo install sewlore-sewing-math`, or build the checkout with `cargo build --release`. The binary is named `sewlore-measure`.

```text
sewlore-measure stretch 10cm 140mm 10.5cm
sewlore-measure print-scale 100mm 95mm 101mm 0.5
sewlore-measure convert 1in mm
```

The stretch command prints one deterministic JSON line:

```json
{"calculation":"stretch","stretch_percent":40,"residual_change_percent":5,"recovered_extension_percent":87.5,"extension":4,"residual_change":0.5,"length_unit":"cm","unusual_released_length":false}
```

All lengths require an explicit suffix. Mixed units are allowed. Numbers use a dot decimal separator, with scientific notation accepted. Print tolerance defaults to `0.5` percent if omitted. Success exits `0`; invalid input writes a message to stderr, leaves stdout empty, and exits `2`. Undefined recovered extension is JSON `null`. No measurements are saved, sent, or read from a remote service.

## Formulas and interpretation

| Result | Formula | Interpretation |
| --- | --- | --- |
| Stretch | `(stretched - original) / original × 100` | Applied extension relative to the original length. |
| Signed residual change | `(released - original) / original × 100` | Positive is longer after release; negative is shorter. |
| Recovered extension | `(stretched - released) / (stretched - original) × 100` | Fraction of applied extension recovered; undefined when extension is zero. |
| X/Y scale | `measured axis / target × 100` | Separate ratios; neither axis is averaged away. |
| Signed X/Y error | `(measured axis - target) / target × 100` | Negative is undersized; positive is oversized. |
| Axis difference | `abs(X error - Y error)` | Difference in percentage points, compared with the supplied tolerance. |

Stretch requires positive original, stretched, and released lengths, with stretched at least original. Released readings outside the original-to-stretched range are preserved and flagged. Recovery can be negative or over 100%; it is never clipped. Print scale requires positive target, width, and height: a zero target cannot be a denominator. Conversion accepts zero.

`within_tolerance` requires both absolute axis errors to pass. `non_uniform` compares the axis difference independently, so both axes can pass their individual tolerance yet differ from one another by more than that tolerance. Floating-point boundary checks allow `64 × f64::EPSILON × max(100, X scale, Y scale, tolerance)` slack. Returned percentages are not rounded to make a failed comparison pass.

## Units and numeric limits

One inch is exactly 25.4 mm; one centimetre is 10 mm. Functions normalize mixed inputs to the original unit (stretch) or target unit (print scale). Conversion divides the unit factors before multiplying, avoiding an unnecessary overflowing intermediate millimetre value. Non-finite results and conversions that turn a positive value into zero by underflow return `Error::NumericRange`. The library uses ordinary `f64` arithmetic; it does not provide exact decimal arithmetic or measurement uncertainty estimates.

The API computes ratios from caller-supplied readings. It does not establish a material's stretch specification, an accepted testing protocol, printer calibration, or a rescale recommendation. Repeatable physical measurements still need a documented procedure and suitable equipment.

## Development

```text
cargo test
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo doc --no-deps
cargo run --example readings
cargo package
```

Tests cover the published npm package's arithmetic examples, mixed-unit combinations, signed out-of-range released readings, zero-extension recovery, tolerance boundaries, invalid CLI input, and extreme numeric ranges. The crate has no runtime, development, or build dependencies and forbids unsafe Rust. The manifest's Rust requirement is 1.74; the release preparation was tested with Rust 1.94.1 on Windows GNU. Rust 1.74 has not been tested independently.

## License and origin

MIT, copyright 2026 Sewlore. Formula definitions and tolerance handling are adapted from `@sewingselami/sewlore-sewing-math` 0.1.0 (MIT); this crate adds typed units, Rust errors, and a local CLI. See [NOTICE](NOTICE) and [LICENSE](LICENSE). Attribution does not require a backlink in a user's generated results.

