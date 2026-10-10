//! Arithmetic, domain, and unit-invariance regression tests.
use sewlore_sewing_math::{print_scale, stretch, Error, Length, Unit};

fn cm(value: f64) -> Length {
    Length::new(value, Unit::Centimetre).unwrap()
}
fn close(actual: f64, expected: f64) {
    let tolerance = 1e-10 * expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn separate_denominators_match_published_npm_example() {
    let r = stretch(cm(10.0), cm(15.0), cm(10.5)).unwrap();
    close(r.stretch_percent, 50.0);
    close(r.residual_change_percent, 5.0);
    close(r.recovered_extension_percent.unwrap(), 90.0);
    close(r.extension, 5.0);
    close(r.residual_change, 0.5);
    assert_eq!(r.length_unit, Unit::Centimetre);
    assert!(!r.unusual_released_length);
}

#[test]
fn recovery_zero_full_none_and_unusual_are_distinct() {
    for (released, recovery, residual, unusual) in [
        (10.0, 100.0, 0.0, false),
        (15.0, 0.0, 50.0, false),
        (9.5, 110.0, -5.0, true),
        (16.0, -20.0, 60.0, true),
    ] {
        let r = stretch(cm(10.0), cm(15.0), cm(released)).unwrap();
        close(r.recovered_extension_percent.unwrap(), recovery);
        close(r.residual_change_percent, residual);
        assert_eq!(r.unusual_released_length, unusual);
    }
    for released in [9.5, 10.0, 11.0] {
        let r = stretch(cm(10.0), cm(10.0), cm(released)).unwrap();
        assert_eq!(r.recovered_extension_percent, None);
        assert_eq!(r.stretch_percent, 0.0);
        assert_eq!(r.unusual_released_length, released != 10.0);
    }
}

#[test]
fn private_type_rejects_negative_and_non_finite_lengths() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            Length::new(value, Unit::Millimetre),
            Err(Error::NonFinite("length"))
        );
    }
    for value in [-1.0, -f64::MIN_POSITIVE] {
        assert_eq!(Length::new(value, Unit::Inch), Err(Error::NegativeLength));
    }
    assert_eq!(cm(-0.0).value().to_bits(), 0.0_f64.to_bits());
}

#[test]
fn zero_lengths_are_valid_to_convert_but_not_in_relative_readings() {
    let zero = cm(0.0);
    assert_eq!(zero.to(Unit::Inch).unwrap().value(), 0.0);
    assert_eq!(
        stretch(zero, cm(15.0), cm(10.0)),
        Err(Error::ZeroLength("original length"))
    );
    assert_eq!(
        stretch(cm(10.0), zero, cm(10.0)),
        Err(Error::ZeroLength("stretched length"))
    );
    assert_eq!(
        stretch(cm(10.0), cm(15.0), zero),
        Err(Error::ZeroLength("released length"))
    );
    assert_eq!(
        print_scale(zero, cm(10.0), cm(10.0), 0.5),
        Err(Error::ZeroLength("target side"))
    );
    assert_eq!(
        print_scale(cm(10.0), zero, cm(10.0), 0.5),
        Err(Error::ZeroLength("measured width"))
    );
    assert_eq!(
        print_scale(cm(10.0), cm(10.0), zero, 0.5),
        Err(Error::ZeroLength("measured height"))
    );
}

#[test]
fn stretched_below_original_is_rejected_after_normalization() {
    let short = Length::new(99.0, Unit::Millimetre).unwrap();
    assert_eq!(
        stretch(cm(10.0), short, cm(10.0)),
        Err(Error::StretchedBelowOriginal)
    );
}

#[test]
fn known_unit_equivalences_and_round_trips_are_correct() {
    close(
        Length::new(1.0, Unit::Inch)
            .unwrap()
            .to(Unit::Millimetre)
            .unwrap()
            .value(),
        25.4,
    );
    close(
        Length::new(25.4, Unit::Millimetre)
            .unwrap()
            .to(Unit::Inch)
            .unwrap()
            .value(),
        1.0,
    );
    for from in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
        for to in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
            for value in [0.0, 0.001, 1.0, 2.54, 10.0, 300.0, 1e8] {
                let original = Length::new(value, from).unwrap();
                close(original.to(to).unwrap().to(from).unwrap().value(), value);
                assert_eq!(original.to(to).unwrap().unit(), to);
            }
        }
    }
    for (text, expected) in [
        ("mm", Unit::Millimetre),
        ("cm", Unit::Centimetre),
        ("in", Unit::Inch),
    ] {
        assert_eq!(text.parse::<Unit>().unwrap(), expected);
        assert_eq!(expected.symbol(), text);
    }
    for invalid in ["", "m", "inch", "CM", " mm", "mm "] {
        assert_eq!(invalid.parse::<Unit>(), Err(Error::UnknownUnit));
    }
}

#[test]
fn mixed_units_preserve_stretch_percentages_across_27_input_combinations() {
    let expected = stretch(cm(10.0), cm(14.2), cm(10.3)).unwrap();
    for original_unit in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
        for stretched_unit in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
            for released_unit in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
                let r = stretch(
                    cm(10.0).to(original_unit).unwrap(),
                    cm(14.2).to(stretched_unit).unwrap(),
                    cm(10.3).to(released_unit).unwrap(),
                )
                .unwrap();
                close(r.stretch_percent, expected.stretch_percent);
                close(r.residual_change_percent, expected.residual_change_percent);
                close(
                    r.recovered_extension_percent.unwrap(),
                    expected.recovered_extension_percent.unwrap(),
                );
                assert_eq!(r.unusual_released_length, expected.unusual_released_length);
                assert_eq!(r.length_unit, original_unit);
            }
        }
    }
}

#[test]
fn signed_print_scale_and_unequal_axes_match_npm_example() {
    let r = print_scale(cm(10.0), cm(9.5), cm(10.1), 0.5).unwrap();
    close(r.x_scale_percent, 95.0);
    close(r.y_scale_percent, 101.0);
    close(r.x_error_percent, -5.0);
    close(r.y_error_percent, 1.0);
    close(r.difference_percentage_points, 6.0);
    assert!(r.non_uniform);
    assert!(!r.within_tolerance);
}

#[test]
fn perfect_uniform_undersize_and_threshold_decisions_are_distinct() {
    for (x, y, tolerance, within, non_uniform) in [
        (10.0, 10.0, 0.0, true, false),
        (9.8, 9.8, 0.5, false, false),
        (10.05, 10.0, 0.5, true, false),
        (9.95, 10.0, 0.5, true, false),
        (10.051, 10.0, 0.5, false, true),
        (9.949, 10.0, 0.5, false, true),
        (9.95, 10.05, 0.5, true, true),
    ] {
        let r = print_scale(cm(10.0), cm(x), cm(y), tolerance).unwrap();
        assert_eq!(r.within_tolerance, within, "x={x}, y={y}");
        assert_eq!(r.non_uniform, non_uniform, "x={x}, y={y}");
    }
}

#[test]
fn mixed_units_preserve_print_decisions_across_27_input_combinations() {
    let expected = print_scale(cm(10.0), cm(9.95), cm(10.01), 0.5).unwrap();
    for target_unit in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
        for x_unit in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
            for y_unit in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
                let r = print_scale(
                    cm(10.0).to(target_unit).unwrap(),
                    cm(9.95).to(x_unit).unwrap(),
                    cm(10.01).to(y_unit).unwrap(),
                    0.5,
                )
                .unwrap();
                close(r.x_error_percent, expected.x_error_percent);
                close(r.y_error_percent, expected.y_error_percent);
                close(
                    r.difference_percentage_points,
                    expected.difference_percentage_points,
                );
                assert_eq!(r.within_tolerance, expected.within_tolerance);
                assert_eq!(r.non_uniform, expected.non_uniform);
            }
        }
    }
}

#[test]
fn tolerance_domain_is_finite_non_negative() {
    assert_eq!(
        print_scale(cm(10.0), cm(10.0), cm(10.0), -0.1),
        Err(Error::NegativeTolerance)
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            print_scale(cm(10.0), cm(10.0), cm(10.0), invalid),
            Err(Error::NonFinite("tolerance percent"))
        );
    }
    assert_eq!(
        print_scale(cm(10.0), cm(10.0), cm(10.0), -0.0)
            .unwrap()
            .tolerance_percent
            .to_bits(),
        0.0_f64.to_bits()
    );
    assert!(
        print_scale(cm(10.0), cm(10.0), cm(10.0), f64::MAX)
            .unwrap()
            .within_tolerance
    );
}

#[test]
fn extreme_ranges_never_return_non_finite_results() {
    let tiny = cm(f64::from_bits(1));
    let large = cm(f64::MAX);
    assert_eq!(stretch(tiny, large, cm(1.0)), Err(Error::NumericRange));
    assert_eq!(
        print_scale(tiny, cm(1.0), cm(1.0), 0.5),
        Err(Error::NumericRange)
    );
    assert_eq!(large.to(Unit::Millimetre), Err(Error::NumericRange));
    assert_eq!(
        Length::new(f64::from_bits(1), Unit::Millimetre)
            .unwrap()
            .to(Unit::Inch),
        Err(Error::NumericRange)
    );
    // Same-unit values avoid an unnecessary conversion to overflowing mm.
    assert_eq!(stretch(large, large, large).unwrap().stretch_percent, 0.0);
    assert!(
        print_scale(large, large, large, 0.0)
            .unwrap()
            .within_tolerance
    );
}

#[test]
fn output_errors_are_actionable_and_implement_std_error() {
    let e: &dyn std::error::Error = &Error::ZeroLength("target side");
    assert_eq!(e.to_string(), "target side must be greater than zero");
    assert!(!Error::NumericRange.to_string().is_empty());
}
