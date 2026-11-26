fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--session"] {
        if let Err(code) = dk_gamesound::prepare_engine_session() {
            eprintln!("{code}");
            std::process::exit(1);
        }
    } else if !args.is_empty() {
        eprintln!("invalid_arguments");
        std::process::exit(1);
    }
    let result = dk_gamesound::runtime()
        .map_err(|_| "runtime_unavailable")
        .and_then(|runtime| runtime.block_on(dk_gamesound::run_engine()));
    if let Err(code) = result {
        eprintln!("{code}");
        std::process::exit(1);
    }
}
