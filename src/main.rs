//! Entry point — Highlander Forge Blade

use highlander_forge_blade::config;
use highlander_forge_blade::logging::{self, LogFormat};
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let log_format = if args.headless { LogFormat::Json } else { LogFormat::Human };
    logging::init_logging(log_format);

    tracing::info!("Highlander Forge Blade v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Modo: {}", if args.headless { "headless" } else { "tui" });

    #[cfg(windows)]
    if !is_admin() {
        tracing::error!("Este aplicativo requer privilegios de Administrador!");
        eprintln!("ERRO: Execute como Administrador.");
        process::exit(1);
    }

    let _config = config::load_or_default();

    if args.headless {
        run_headless(args).await
    } else {
        #[cfg(feature = "tui")]
        { run_tui(args).await }
        #[cfg(not(feature = "tui"))]
        {
            tracing::error!("Feature 'tui' nao compilada. Use --features tui");
            process::exit(1);
        }
    }
}

#[cfg(feature = "tui")]
async fn run_tui(_args: Args) -> Result<(), Box<dyn std::error::Error>> {
    use highlander_forge_blade::ui::ratatui;
    ratatui::run().await
}

async fn run_headless(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    use highlander_forge_blade::app::headless;
    headless::run(args.auto_phase, args.what_if).await
}

#[derive(Debug, Clone)]
struct Args {
    headless: bool,
    auto_phase: Option<String>,
    what_if: bool,
    check_update: bool,
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat { Human, Json }

fn parse_args() -> Args {
    let mut args = Args { headless: false, auto_phase: None, what_if: false, check_update: false, format: OutputFormat::Human };
    let raw: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < raw.len() {
        match raw[i].as_str() {
            "--auto-phase" => { i += 1; if i < raw.len() { args.headless = true; args.auto_phase = Some(raw[i].clone()); } }
            "--what-if" => args.what_if = true,
            "--check-update" => { args.check_update = true; args.headless = true; }
            "--format" => { i += 1; if i < raw.len() { args.format = match raw[i].as_str() { "json" => OutputFormat::Json, _ => OutputFormat::Human }; } }
            "--headless" => args.headless = true,
            _ => {}
        }
        i += 1;
    }
    args
}

#[cfg(windows)]
fn is_admin() -> bool {
    use windows::Win32::Foundation::{BOOL, PSID};
    use windows::Win32::Security::{AllocateAndInitializeSid, CheckTokenMembership, FreeSid, SID_IDENTIFIER_AUTHORITY};
    unsafe {
        let authority = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] };
        let mut sid: PSID = PSID(std::ptr::null_mut());
        if AllocateAndInitializeSid(&authority, 2, 32, 544, 0, 0, 0, 0, 0, 0, &mut sid).is_err() {
            return false;
        }
        let mut member = BOOL(0);
        let _ = CheckTokenMembership(None, sid, &mut member);
        let _ = FreeSid(sid);
        member.as_bool()
    }
}

#[cfg(not(windows))]
fn is_admin() -> bool { true }
