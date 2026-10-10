//! Native CLI exit codes, JSON values, parsing, and stderr regression tests.
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sewlore-measure"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn cli_stretch_json_has_known_exact_readings_and_stable_order() {
    let r = run(&["stretch", "10cm", "140mm", "10.5cm"]);
    assert!(r.status.success());
    assert!(r.stderr.is_empty());
    assert_eq!(String::from_utf8(r.stdout).unwrap(), "{\"calculation\":\"stretch\",\"stretch_percent\":40,\"residual_change_percent\":5,\"recovered_extension_percent\":87.5,\"extension\":4,\"residual_change\":0.5,\"length_unit\":\"cm\",\"unusual_released_length\":false}\n");
}

#[test]
fn undefined_recovery_is_json_null_and_unusual_reading_is_preserved() {
    let r = run(&["stretch", "10cm", "10cm", "9.5cm"]);
    assert!(r.status.success());
    let text = String::from_utf8(r.stdout).unwrap();
    assert!(text.contains("\"recovered_extension_percent\":null"));
    assert!(text.contains("\"residual_change_percent\":-5"));
    assert!(text.contains("\"unusual_released_length\":true"));
}

#[test]
fn print_scale_keeps_signed_axes_and_default_tolerance() {
    let r = run(&["print-scale", "100mm", "95mm", "101mm"]);
    assert!(r.status.success());
    assert!(r.stderr.is_empty());
    let text = String::from_utf8(r.stdout).unwrap();
    for part in [
        "\"x_error_percent\":-5",
        "\"y_error_percent\":1",
        "\"difference_percentage_points\":6",
        "\"non_uniform\":true",
        "\"within_tolerance\":false",
        "\"tolerance_percent\":0.5",
    ] {
        assert!(text.contains(part), "missing {part}: {text}");
    }
}

#[test]
fn conversion_supports_scientific_notation_and_zero() {
    for (input, unit, expected) in [
        ("1in", "mm", "25.4"),
        ("1e1cm", "mm", "100"),
        ("-0cm", "in", "0"),
    ] {
        let r = run(&["convert", input, unit]);
        assert!(r.status.success());
        assert!(r.stderr.is_empty());
        assert_eq!(
            String::from_utf8(r.stdout).unwrap(),
            format!("{{\"calculation\":\"convert\",\"value\":{expected},\"unit\":\"{unit}\"}}\n")
        );
    }
}

#[test]
fn invalid_cli_input_has_no_json_or_stdout_and_exits_two() {
    for args in [
        vec![],
        vec!["unknown"],
        vec!["stretch", "10", "15cm", "10cm"],
        vec!["stretch", "0cm", "15cm", "10cm"],
        vec!["stretch", "10cm", "9cm", "10cm"],
        vec!["stretch", "NaNcm", "15cm", "10cm"],
        vec!["stretch", "infcm", "15cm", "10cm"],
        vec!["stretch", "10,5cm", "15cm", "10cm"],
        vec!["convert", "-1in", "cm"],
        vec!["convert", "1cm", "m"],
        vec!["print-scale", "0mm", "100mm", "100mm"],
        vec!["print-scale", "100mm", "100mm", "100mm", "-1"],
        vec!["print-scale", "100mm", "100mm", "100mm", "NaN"],
        vec!["convert", "1cm", "mm", "extra"],
    ] {
        let r = run(&args);
        assert_eq!(r.status.code(), Some(2), "{args:?}");
        assert!(r.stdout.is_empty(), "{args:?}");
        assert!(
            String::from_utf8(r.stderr).unwrap().starts_with("error: "),
            "{args:?}"
        );
    }
}

#[test]
fn help_and_version_are_available_offline() {
    let r = run(&["--help"]);
    assert!(r.status.success());
    assert!(r.stderr.is_empty());
    assert!(String::from_utf8(r.stdout)
        .unwrap()
        .contains("No network or persistent storage"));
    let version = run(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "sewlore-measure 0.1.0\n"
    );
}
