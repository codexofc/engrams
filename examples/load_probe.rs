//! Load time of the full model and encoding time of a short query.
fn main() {
    let dir = engrams::paths::model_dir();
    for _ in 0..3 {
        let t = std::time::Instant::now();
        let e = engrams::embedder::Embedder::load(&dir).unwrap();
        let load = t.elapsed();
        let t = std::time::Instant::now();
        let _ = e.encode("how are the databases isolated between agents").unwrap();
        println!("load {:>4} ms, encode one query {:>3} ms", load.as_millis(), t.elapsed().as_millis());
    }
}
