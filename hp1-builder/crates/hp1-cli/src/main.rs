use hp1_core::{build, inspect_zip, load_profiles, BuildEvent, BuildRequest, DisplayProfile, EventSink};
use std::path::PathBuf;

const EXIT_USAGE: i32 = 2;
const EXIT_INPUT: i32 = 3;
const EXIT_UNSUPPORTED: i32 = 4;
const EXIT_INTERNAL: i32 = 5;

fn main() {
    if let Err((code, message)) = run(std::env::args().skip(1).collect()) {
        eprintln!("hp1-builder: {message}");
        std::process::exit(code);
    }
}

fn run(args: Vec<String>) -> Result<(), (i32, String)> {
    let mut args = args.into_iter();
    let command = args.next().unwrap_or_else(|| "--help".into());
    match command.as_str() {
        "--help" | "-h" | "help" => { print_help(); Ok(()) }
        "inspect" => {
            let (zip, profiles, json) = parse_inspect(args.collect())?;
            let inspection = inspect_zip(&zip).map_err(core_error)?;
            let matched = load_profiles(&profiles).map_err(|e| (EXIT_INTERNAL, e.to_string()))?
                .into_iter().find(|p| p.matches(&inspection));
            if json {
                let profile = matched.map(|p| format!("\"{}\"", json_escape(&p.id))).unwrap_or_else(|| "null".into());
                println!("{{\"sha256\":\"{}\",\"size\":{},\"supported_profile\":{profile}}}", inspection.sha256, inspection.size);
            } else {
                let profile = matched.map(|p| format!("{} ({})", p.edition_name, p.region)).unwrap_or_else(|| "unsupported".into());
                println!("ZIP SHA-256: {}\nSize: {} bytes\nProfile: {profile}", inspection.sha256, inspection.size);
            }
            Ok(())
        }
        "build" => {
            let (request, json) = parse_build(args.collect())?;
            let mut sink = Printer { json };
            build(&request, &mut sink).map_err(core_error)
        }
        "check" => check(),
        _ => Err((EXIT_USAGE, format!("unknown command {command:?}; run hp1-builder --help"))),
    }
}

fn default_profiles() -> PathBuf { PathBuf::from("profiles") }

fn parse_inspect(arguments: Vec<String>) -> Result<(PathBuf, PathBuf, bool), (i32, String)> {
    let mut iso = None; let mut profiles = default_profiles(); let mut json = false; let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--zip" => { index += 1; iso = Some(PathBuf::from(arguments.get(index).ok_or((EXIT_USAGE, "--zip needs a path".into()))?)); }
            "--profiles" => { index += 1; profiles = PathBuf::from(arguments.get(index).ok_or((EXIT_USAGE, "--profiles needs a directory".into()))?); }
            "--log-format" => { index += 1; json = parse_log_format(arguments.get(index).ok_or((EXIT_USAGE, "--log-format needs text or json".into()))?)?; }
            value => return Err((EXIT_USAGE, format!("unknown inspect argument {value:?}"))),
        }
        index += 1;
    }
    Ok((iso.ok_or((EXIT_USAGE, "inspect requires --zip PATH".into()))?, profiles, json))
}

fn parse_build(arguments: Vec<String>) -> Result<(BuildRequest, bool), (i32, String)> {
    let mut zip = None; let mut output = None; let mut profiles = default_profiles();
    let mut default_profile = DisplayProfile::Hd720;
    let mut dry_run = false; let mut keep_workdir = false; let mut json = false; let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--zip" => { index += 1; zip = Some(PathBuf::from(arguments.get(index).ok_or((EXIT_USAGE, "--zip needs a path".into()))?)); }
            "--output" => { index += 1; output = Some(PathBuf::from(arguments.get(index).ok_or((EXIT_USAGE, "--output needs a directory".into()))?)); }
            "--profiles" => { index += 1; profiles = PathBuf::from(arguments.get(index).ok_or((EXIT_USAGE, "--profiles needs a directory".into()))?); }
            "--profile" => { index += 1; default_profile = DisplayProfile::from_id(arguments.get(index).ok_or((EXIT_USAGE, "--profile needs a value".into()))?).ok_or((EXIT_USAGE, "--profile must be 720p, 1080p, deck, ultrawide or windowed".into()))?; }
            "--dry-run" => dry_run = true,
            "--keep-workdir" => keep_workdir = true,
            "--log-format" => { index += 1; json = parse_log_format(arguments.get(index).ok_or((EXIT_USAGE, "--log-format needs text or json".into()))?)?; }
            "--report" => { index += 1; let _ = arguments.get(index).ok_or((EXIT_USAGE, "--report needs a path".into()))?; }
            "--non-interactive" => {}
            value => return Err((EXIT_USAGE, format!("unknown build argument {value:?}"))),
        }
        index += 1;
    }
    Ok((BuildRequest { source_zip: zip.ok_or((EXIT_USAGE, "build requires --zip PATH".into()))?, output: output.ok_or((EXIT_USAGE, "build requires --output DIRECTORY".into()))?, profiles_dir: profiles, dry_run, keep_workdir, default_profile }, json))
}

fn parse_log_format(value: &str) -> Result<bool, (i32, String)> { match value { "text" => Ok(false), "json" => Ok(true), _ => Err((EXIT_USAGE, "--log-format accepts text or json".into())) } }
fn check() -> Result<(), (i32, String)> { let profiles = load_profiles(&default_profiles()).map_err(|e| (EXIT_INTERNAL, e.to_string()))?; println!("Core checks passed: {} ZIP profile(s) loaded.\nGame ZIPs stay local; free compatibility components are downloaded and SHA-256 verified on the first build.", profiles.len()); Ok(()) }
fn core_error(error: hp1_core::CoreError) -> (i32, String) { let code = match error { hp1_core::CoreError::InvalidInput(_) => EXIT_INPUT, hp1_core::CoreError::Unsupported(_) => EXIT_UNSUPPORTED, _ => EXIT_INTERNAL }; (code, error.to_string()) }
struct Printer { json: bool }
impl EventSink for Printer { fn emit(&mut self, event: BuildEvent) { if self.json { println!("{{\"stage\":\"{}\",\"event\":\"{}\",\"message\":\"{}\"}}", event.stage.as_str(), event.kind, json_escape(&event.message)); } else if event.message.is_empty() { println!("[{}] {}", event.stage.as_str(), event.kind); } else { println!("[{}] {}: {}", event.stage.as_str(), event.kind, event.message); } } }
fn json_escape(input: &str) -> String { input.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n") }
fn print_help() { println!("HP1 Builder — local-only ZIP-to-AppImage builder\n\nUsage:\n  hp1-builder inspect --zip PATH [--profiles DIR] [--log-format text|json]\n  hp1-builder build --zip PATH --output DIRECTORY [--profile 720p|1080p|deck|ultrawide|windowed] [--dry-run] [--keep-workdir] [--profiles DIR] [--log-format text|json]\n  hp1-builder check\n\nThe builder accepts only a verified locally supplied ZIP profile and never downloads game inputs.\n\nExit codes: 0 success; 2 command-line usage; 3 invalid input; 4 unsupported/gated media; 5 internal error."); }
