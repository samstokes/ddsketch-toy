mod ring;
mod sketch;

use crate::{ring::Ring, sketch::Sketch};

fn main() {
    println!("Did you know? log2(0) == {}", 0.0f32.log2());
    println!(
        "Did you know? index_for(0) ~= {}",
        0.0f32.log2().ceil() as i16
    );
    println!("Did you know? log2(-0) == {}", -0.0f32.log2());
    println!(
        "Did you know? index_for(-0) ~= {}",
        -0.0f32.log2().ceil() as i16
    );

    let r = Ring::<()>::new(5);

    println!("ring capacity: {}", r.capacity());

    let mut s = Sketch::new(0.05);
    s.insert(0.3);
    println!("sketch size: {}", s.size());
    println!("sketch min: {}", s.quantile(0.0));
}
