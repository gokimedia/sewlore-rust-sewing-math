//! Arithmetic example using hypothetical readings, not measured material data.
use sewlore_sewing_math::{print_scale, stretch, Length, Unit};

fn main() -> Result<(), sewlore_sewing_math::Error> {
    let s = stretch(
        Length::new(10.0, Unit::Centimetre)?,
        Length::new(140.0, Unit::Millimetre)?,
        Length::new(10.5, Unit::Centimetre)?,
    )?;
    println!(
        "hypothetical stretch={}%, residual={}%, recovery={:?}%",
        s.stretch_percent, s.residual_change_percent, s.recovered_extension_percent
    );
    let p = print_scale(
        Length::new(100.0, Unit::Millimetre)?,
        Length::new(95.0, Unit::Millimetre)?,
        Length::new(10.1, Unit::Centimetre)?,
        0.5,
    )?;
    println!(
        "hypothetical print X={}%, Y={}%, within_tolerance={}, non_uniform={}",
        p.x_error_percent, p.y_error_percent, p.within_tolerance, p.non_uniform
    );
    Ok(())
}
