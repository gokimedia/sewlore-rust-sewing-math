//! Offline JSON command line interface for sewing measurement arithmetic.

use sewlore_sewing_math::{print_scale, stretch, Length, Unit};
use std::env;
use std::process::ExitCode;

const HELP: &str = "sewlore-measure — local sewing measurement arithmetic\n\nUsage:\n  sewlore-measure stretch ORIGINAL STRETCHED RELEASED\n  sewlore-measure print-scale TARGET WIDTH HEIGHT [TOLERANCE_PERCENT]\n  sewlore-measure convert LENGTH UNIT\n  sewlore-measure --help\n  sewlore-measure --version\n\nEach LENGTH must have an explicit mm, cm, or in suffix (e.g. 100mm, 10cm, 4in).\nMixed units are allowed. Numbers use a dot decimal separator; scientific notation\nis accepted. Tolerance defaults to 0.5 percent. Valid results are one JSON line.\nErrors go to stderr and exit 2; success exits 0. No network or persistent storage.\n\nExamples:\n  sewlore-measure stretch 10cm 140mm 10.5cm\n  sewlore-measure print-scale 100mm 99.5mm 100.2mm 0.5\n  sewlore-measure convert 1in mm\n\nArithmetic only: results do not establish fabric properties or printer calibration.\n";

fn length(value: &str) -> Result<Length, String> {
    for unit in [Unit::Millimetre, Unit::Centimetre, Unit::Inch] {
        if let Some(number) = value.strip_suffix(unit.symbol()) {
            let number = number.parse::<f64>().map_err(|_| {
                "length requires a valid number and mm, cm, or in suffix".to_owned()
            })?;
            return Length::new(number, unit).map_err(|e| e.to_string());
        }
    }
    Err("length requires an explicit mm, cm, or in suffix".to_owned())
}

fn evaluate(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("stretch") if args.len() == 4 => {
            let r = stretch(length(&args[1])?, length(&args[2])?, length(&args[3])?)
                .map_err(|e| e.to_string())?;
            let recovery = r
                .recovered_extension_percent
                .map_or_else(|| "null".to_owned(), |n| n.to_string());
            Ok(format!("{{\"calculation\":\"stretch\",\"stretch_percent\":{},\"residual_change_percent\":{},\"recovered_extension_percent\":{},\"extension\":{},\"residual_change\":{},\"length_unit\":\"{}\",\"unusual_released_length\":{}}}", r.stretch_percent, r.residual_change_percent, recovery, r.extension, r.residual_change, r.length_unit.symbol(), r.unusual_released_length))
        }
        Some("print-scale") if args.len() == 4 || args.len() == 5 => {
            let tolerance = if args.len() == 5 {
                args[4]
                    .parse::<f64>()
                    .map_err(|_| "tolerance percent requires a valid number".to_owned())?
            } else {
                0.5
            };
            let r = print_scale(
                length(&args[1])?,
                length(&args[2])?,
                length(&args[3])?,
                tolerance,
            )
            .map_err(|e| e.to_string())?;
            Ok(format!("{{\"calculation\":\"print_scale\",\"x_scale_percent\":{},\"y_scale_percent\":{},\"x_error_percent\":{},\"y_error_percent\":{},\"difference_percentage_points\":{},\"non_uniform\":{},\"within_tolerance\":{},\"tolerance_percent\":{}}}", r.x_scale_percent, r.y_scale_percent, r.x_error_percent, r.y_error_percent, r.difference_percentage_points, r.non_uniform, r.within_tolerance, r.tolerance_percent))
        }
        Some("convert") if args.len() == 3 => {
            let unit = args[2].parse::<Unit>().map_err(|e| e.to_string())?;
            let r = length(&args[1])?.to(unit).map_err(|e| e.to_string())?;
            Ok(format!(
                "{{\"calculation\":\"convert\",\"value\":{},\"unit\":\"{}\"}}",
                r.value(),
                r.unit().symbol()
            ))
        }
        _ => Err("unknown command or wrong argument count; run sewlore-measure --help".to_owned()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        print!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if args.len() == 1 && args[0] == "--version" {
        println!("sewlore-measure {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    match evaluate(&args) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
