use extractous::Extractor;
use std::env;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

const BUFSIZ: usize = 8192;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: extractous <input file> <output file>");
        std::process::exit(1);
    }
    let input = &args[1];
    let output = &args[2];

    let extractor = Extractor::new();
    let (mut text_reader, _metadata) = extractor
        .extract_file(input)
        .expect("failed to extract text from file");

    let mut buf: [u8; BUFSIZ] = [0_u8; BUFSIZ];
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)
        .unwrap();

    loop {
        match text_reader.read(&mut buf) {
            Ok(n_read) => {
                output_file.write_all(&buf[..n_read]).unwrap_or_else(|e| {
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
