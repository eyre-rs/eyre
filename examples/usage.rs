use human_eyre::{eyre, Report};

fn main() {
    let e: Report = eyre!("some random issue");
    println!("Error: {:?}", e);
}
