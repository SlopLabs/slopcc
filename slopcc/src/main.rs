#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    println!("slopcc: no compiler phases implemented yet");
}
