use coin_io::CoinMetadata;

fn main() {
    gear_wasm_builder::build_with_metadata::<CoinMetadata>();
}
