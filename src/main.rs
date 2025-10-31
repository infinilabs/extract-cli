use extractous::Extractor;
use std::env;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

const BUFSIZ: usize = 8192;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input file>", args[0]);
        std::process::exit(1);
    }
    let input = &args[1];

    let extractor = Extractor::new();
    let (mut text_reader, _metadata) = extractor
        .extract_file(input)
        .expect("failed to extract text from file");
    let mut stdout = std::io::stdout();

    let mut buf: [u8; BUFSIZ] = [0_u8; BUFSIZ];

    loop {
        match text_reader.read(&mut buf) {
            Ok(n_read) => {
                if n_read == 0 {
                    break;
                }

                stdout.write_all(&buf[..n_read]).unwrap_or_else(|e| {
                    eprintln!("Error: I/O error, {}", e);
                    std::process::exit(1);
                });
            }
            Err(e) => {
                let error_kind = e.kind();
                if error_kind == ErrorKind::Interrupted {
                    // retry
                    continue;
                } else {
                    eprintln!("Error: I/O error, {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
