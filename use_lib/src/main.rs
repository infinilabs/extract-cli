use extractous::Extractor;

fn main() {
  // Get the command-line arguments
  let args: Vec<String> = std::env::args().collect();
  let file_path = &args[1];

  // Extract the provided file content to a string
  let mut extractor = Extractor::new();
  // if you need an xml
  // extractor = extractor.set_xml_output(false);
  // Extract text from a file
  let (content, metadata) = extractor.extract_file_to_string(file_path).unwrap();
  println!("{}", content);
  println!("{:?}", metadata);
}