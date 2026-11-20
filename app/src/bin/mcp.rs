use dk_gamesound::{
    mcp::{parse_error, Session},
    protocol::MAX_FRAME,
};
use std::io::{self, BufRead, Read, Write};

fn main() -> io::Result<()> {
    let runtime = dk_gamesound::runtime()?;
    let mut session = Session::default();
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    loop {
        let mut bytes = Vec::new();
        let size = stdin
            .by_ref()
            .take(MAX_FRAME as u64 + 1)
            .read_until(b'\n', &mut bytes)?;
        if size == 0 {
            break;
        }
        if size > MAX_FRAME {
            writeln!(stdout, "{}", parse_error())?;
            stdout.flush()?;
            break;
        }
        let response = match serde_json::from_slice(&bytes) {
            Ok(value) => runtime.block_on(session.handle(value)),
            Err(_) => Some(parse_error()),
        };
        if let Some(value) = response {
            writeln!(stdout, "{value}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}
