fn main() {
    let result = dk_gamesound::runtime()
        .map_err(|_| "runtime_unavailable")
        .and_then(|runtime| runtime.block_on(dk_gamesound::run_engine()));
    if let Err(code) = result {
        eprintln!("{code}");
        std::process::exit(1);
    }
}
