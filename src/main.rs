#![feature(iter_array_chunks)]
use ecc::gen_keys;
use encoding_utils::{points_to_text, text_to_points};

mod ecc;
mod encoding_utils;
mod field;

fn main() {
    let mut rng = rand::thread_rng();
    let (pr, pb) = gen_keys(&mut rng);
    let msg = "Hello, world!";
    println!("pubkey: {:?}", pb);
    println!("private: {:?}", pr);
    println!("MSG: {:?}", msg);
    let points = text_to_points(msg);
    println!("MSG p: {:?}", points);

    let encrypted = points
        .iter()
        .map(|p| pb.encrypt(*p, &mut rng))
        .collect::<Vec<_>>();
    println!("Encrypted: {:?}", encrypted);

    let decryped = encrypted.iter().map(|p| pr.decrypt(*p)).collect::<Vec<_>>();
    println!("Decrypted: {:?}", decryped);

    println!("MSG: {:?}", points_to_text(&decryped));
    // println!("{:?}", Zp::new(19381031) / Zp::new(312983120));
    // println!("Hello, world! {:?}", gen_keys());
}

#[cfg(test)]
mod tests {

    use crate::{points_to_text, text_to_points};

    #[test]
    fn text2points2text() {
        let text = "Hello, world";
        let points = text_to_points(text);
        let text2 = points_to_text(&points);
        assert_eq!(text, text2);
    }
}
