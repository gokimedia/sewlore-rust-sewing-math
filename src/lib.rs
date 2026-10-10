//! Sewing measurement arithmetic with explicit length units and finite results.
//!
//! This crate operates on readings supplied by the caller. It does not measure
//! fabric, infer material properties, validate a printer, or prescribe a print
//! correction. All functions are local and have no external dependencies.
//! Methods and worked examples: [Sewlore Rust sewing measurements](https://sewlore.com/pages/rust-sewing-measurements).
//!
//! Stretch and residual change use the **original length** as denominator;
//! recovered extension uses the **applied extension**. That fraction is
//! undefined when the extension is zero, represented by [`Option::None`].
//! Released readings outside the original-to-stretched interval remain visible
//! and are flagged rather than clamped.
//!
//! # Example
//! ```
//! use sewlore_sewing_math::{stretch, Length, Unit};
//! let reading = stretch(
//!     Length::new(10.0, Unit::Centimetre)?,
//!     Length::new(140.0, Unit::Millimetre)?,
//!     Length::new(10.5, Unit::Centimetre)?,
//! )?;
//! assert_eq!(reading.stretch_percent, 40.0);
//! assert_eq!(reading.residual_change_percent, 5.0);
//! assert_eq!(reading.recovered_extension_percent, Some(87.5));
//! # Ok::<(), sewlore_sewing_math::Error>(())
//! ```
//!
//! Mixed inputs are normalized to the original unit for stretch or the target
//! unit for print scale. This avoids unnecessary conversion to millimetres when
//! large same-unit readings can already be evaluated. Overflow and conversions
//! that underflow a positive input to zero return [`Error::NumericRange`].
//! Ordinary binary floating-point rounding remains; this is not exact decimal
//! arithmetic or a statement of measurement uncertainty.

use std::fmt;
use std::str::FromStr;

/// Supported explicit length units. One inch is exactly 25.4 millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Millimetres (`mm`).
    Millimetre,
    /// Centimetres (`cm`).
    Centimetre,
    /// Inches (`in`).
    Inch,
}

impl Unit {
    /// Return the CLI symbol for this unit.
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Millimetre => "mm",
            Self::Centimetre => "cm",
            Self::Inch => "in",
        }
    }

    fn millimetre_factor(self) -> f64 {
        match self {
            Self::Millimetre => 1.0,
            Self::Centimetre => 10.0,
            Self::Inch => 25.4,
        }
    }
}

impl FromStr for Unit {
    type Err = Error;

    /// Parse exactly `mm`, `cm`, or `in`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mm" => Ok(Self::Millimetre),
            "cm" => Ok(Self::Centimetre),
            "in" => Ok(Self::Inch),
            _ => Err(Error::UnknownUnit),
        }
    }
}

/// A finite, non-negative length. Fields are private to preserve this invariant.
///
/// Zero is allowed for conversion. Stretch and print-scale inputs require
/// positive lengths and reject zero with a named [`Error::ZeroLength`].
/// ```
/// use sewlore_sewing_math::{Length, Unit};
/// let inch = Length::new(1.0, Unit::Inch)?;
/// assert_eq!(inch.to(Unit::Millimetre)?.value(), 25.4);
/// # Ok::<(), sewlore_sewing_math::Error>(())
/// ```
/// ```compile_fail
/// use sewlore_sewing_math::{Length, Unit};
/// let invalid = Length { value: -1.0, unit: Unit::Millimetre };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Length {
    value: f64,
    unit: Unit,
}

impl Length {
    /// Construct a length; reject negatives, NaN, and infinity.
    /// Negative zero is normalized to positive zero.
    pub fn new(value: f64, unit: Unit) -> Result<Self, Error> {
        finite(value, "length")?;
        if value < 0.0 {
            return Err(Error::NegativeLength);
        }
        Ok(Self {
            value: zero(value),
            unit,
        })
    }

    /// Return the finite length value in [`Self::unit`].
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Return the unit associated with this value.
    pub const fn unit(self) -> Unit {
        self.unit
    }

    /// Convert to an explicit unit. Divide unit factors before multiplying to
    /// avoid an unnecessary overflowing intermediate millimetre value.
    pub fn to(self, unit: Unit) -> Result<Self, Error> {
        if unit == self.unit {
            return Ok(self);
        }
        let value = self.value * (self.unit.millimetre_factor() / unit.millimetre_factor());
        if !value.is_finite() || (self.value > 0.0 && value == 0.0) {
            return Err(Error::NumericRange);
        }
        Self::new(value, unit)
    }
}

/// Invalid input or an unrepresentable finite result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A named floating-point input was NaN or infinite.
    NonFinite(&'static str),
    /// A physical length was negative.
    NegativeLength,
    /// A named baseline or reading was zero when a positive value was required.
    ZeroLength(&'static str),
    /// The stretched reading was less than the original reading.
    StretchedBelowOriginal,
    /// Print tolerance was negative.
    NegativeTolerance,
    /// A conversion or calculation exceeded the representable numeric range.
    NumericRange,
    /// A unit string was not exactly `mm`, `cm`, or `in`.
    UnknownUnit,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite(name) => write!(f, "{name} must be finite"),
            Self::NegativeLength => f.write_str("length must be non-negative"),
            Self::ZeroLength(name) => write!(f, "{name} must be greater than zero"),
            Self::StretchedBelowOriginal => {
                f.write_str("stretched length cannot be less than original length")
            }
            Self::NegativeTolerance => f.write_str("tolerance percent must be non-negative"),
            Self::NumericRange => {
                f.write_str("numeric range is too large or too small; recheck the readings")
            }
            Self::UnknownUnit => f.write_str("unit must be mm, cm, or in"),
        }
    }
}

impl std::error::Error for Error {}

fn finite(value: f64, field: &'static str) -> Result<f64, Error> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::NonFinite(field))
    }
}

fn result(value: f64) -> Result<f64, Error> {
    if value.is_finite() {
        Ok(zero(value))
    } else {
        Err(Error::NumericRange)
    }
}

fn positive(length: Length, field: &'static str) -> Result<(), Error> {
    if length.value == 0.0 {
        Err(Error::ZeroLength(field))
    } else {
        Ok(())
    }
}

fn zero(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

/// Stretch, signed residual change, and recovered extension from three readings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stretch {
    /// `(stretched - original) / original * 100`.
    pub stretch_percent: f64,
    /// `(released - original) / original * 100`; may be negative.
    pub residual_change_percent: f64,
    /// `(stretched - released) / (stretched - original) * 100`.
    /// `None` if the applied extension is zero; may be outside 0–100.
    pub recovered_extension_percent: Option<f64>,
    /// Applied extension in [`Self::length_unit`].
    pub extension: f64,
    /// Signed released-minus-original change in [`Self::length_unit`].
    pub residual_change: f64,
    /// Original unit used for the absolute output differences.
    pub length_unit: Unit,
    /// True when released is less than original or greater than stretched.
    pub unusual_released_length: bool,
}

/// Evaluate positive original, stretched, and released readings.
///
/// `stretched >= original` is required after unit conversion. An unusual
/// released reading is accepted and flagged, not hidden or clamped.
/// ```
/// use sewlore_sewing_math::{stretch, Length, Unit};
/// let cm = |n| Length::new(n, Unit::Centimetre);
/// let no_extension = stretch(cm(10.0)?, cm(10.0)?, cm(10.0)?)?;
/// assert_eq!(no_extension.recovered_extension_percent, None);
/// let unusual = stretch(cm(10.0)?, cm(15.0)?, cm(9.5)?)?;
/// assert!(unusual.unusual_released_length);
/// assert!((unusual.recovered_extension_percent.unwrap() - 110.0).abs() < 1e-10);
/// # Ok::<(), sewlore_sewing_math::Error>(())
/// ```
pub fn stretch(original: Length, stretched: Length, released: Length) -> Result<Stretch, Error> {
    positive(original, "original length")?;
    positive(stretched, "stretched length")?;
    positive(released, "released length")?;
    let stretched = stretched.to(original.unit)?.value;
    let released = released.to(original.unit)?.value;
    if stretched < original.value {
        return Err(Error::StretchedBelowOriginal);
    }
    let extension = result(stretched - original.value)?;
    let residual = result(released - original.value)?;
    Ok(Stretch {
        stretch_percent: result((extension / original.value) * 100.0)?,
        residual_change_percent: result((residual / original.value) * 100.0)?,
        recovered_extension_percent: if extension == 0.0 {
            None
        } else {
            Some(result(((stretched - released) / extension) * 100.0)?)
        },
        extension,
        residual_change: residual,
        length_unit: original.unit,
        unusual_released_length: released < original.value || released > stretched,
    })
}

/// Signed, independent width and height comparison against one target side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrintScale {
    /// Measured width / target * 100.
    pub x_scale_percent: f64,
    /// Measured height / target * 100.
    pub y_scale_percent: f64,
    /// Width-minus-target / target * 100; negative is undersized.
    pub x_error_percent: f64,
    /// Height-minus-target / target * 100; negative is undersized.
    pub y_error_percent: f64,
    /// Absolute X-minus-Y error in percentage points.
    pub difference_percentage_points: f64,
    /// X/Y difference exceeds the chosen tolerance, allowing floating rounding.
    pub non_uniform: bool,
    /// Both absolute errors are within the chosen tolerance.
    pub within_tolerance: bool,
    /// Caller-supplied non-negative percentage tolerance.
    pub tolerance_percent: f64,
}

/// Compare two positive measured sides against one positive target side.
///
/// A zero target is rejected because relative error needs a non-zero
/// denominator. Tolerance is a finite non-negative percentage, not a length.
/// Boundary comparisons allow `64 * f64::EPSILON * max(100, X scale,
/// Y scale, tolerance)` slack without rounding the returned measurements.
/// This is a comparison, **not a printer calibration or rescale recommendation**.
/// ```
/// use sewlore_sewing_math::{print_scale, Length, Unit};
/// let mm = |n| Length::new(n, Unit::Millimetre);
/// let p = print_scale(mm(100.0)?, mm(95.0)?, mm(101.0)?, 0.5)?;
/// assert_eq!(p.x_error_percent, -5.0);
/// assert_eq!(p.y_error_percent, 1.0);
/// assert!(p.non_uniform);
/// assert!(!p.within_tolerance);
/// # Ok::<(), sewlore_sewing_math::Error>(())
/// ```
pub fn print_scale(
    target: Length,
    measured_x: Length,
    measured_y: Length,
    tolerance_percent: f64,
) -> Result<PrintScale, Error> {
    positive(target, "target side")?;
    positive(measured_x, "measured width")?;
    positive(measured_y, "measured height")?;
    finite(tolerance_percent, "tolerance percent")?;
    if tolerance_percent < 0.0 {
        return Err(Error::NegativeTolerance);
    }
    let x = measured_x.to(target.unit)?.value;
    let y = measured_y.to(target.unit)?.value;
    let x_scale_percent = result((x / target.value) * 100.0)?;
    let y_scale_percent = result((y / target.value) * 100.0)?;
    let x_error_percent = result(((x - target.value) / target.value) * 100.0)?;
    let y_error_percent = result(((y - target.value) / target.value) * 100.0)?;
    let difference_percentage_points = result((x_error_percent - y_error_percent).abs())?;
    let slack = 64.0
        * f64::EPSILON
        * 100.0_f64
            .max(x_scale_percent)
            .max(y_scale_percent)
            .max(tolerance_percent);
    // Compare the excess rather than tolerance + slack; their sum can overflow
    // for a finite maximum tolerance even though the decision is meaningful.
    let exceeds = |value: f64| value > tolerance_percent && value - tolerance_percent > slack;
    Ok(PrintScale {
        x_scale_percent,
        y_scale_percent,
        x_error_percent,
        y_error_percent,
        difference_percentage_points,
        non_uniform: exceeds(difference_percentage_points),
        within_tolerance: !exceeds(x_error_percent.abs()) && !exceeds(y_error_percent.abs()),
        tolerance_percent: zero(tolerance_percent),
    })
}
