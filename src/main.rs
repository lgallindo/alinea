fn main() {
    if let Err(error) = alinea::app::run() {
        eprintln!("Alinea failed to start: {error}");
    }
}
