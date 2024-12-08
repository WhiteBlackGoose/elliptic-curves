#![feature(iter_array_chunks)]
use clap::{Arg, Command};
use ecc::{gen_keys, Point, PrivateKey, PublicKey};
use encoding_utils::{
    decode_message_and_decrypt, encrypt_message_and_encode, points_to_text, text_to_points,
};
use rand::Rng;

mod ecc;
mod encoding_utils;
mod field;

fn main() {
    let matches = Command::new("xxx")
        .subcommand(Command::new("genkey").about("Generate a pair of keys"))
        .subcommand(
            Command::new("encrypt")
                .about("Encrypt a message")
                .arg(Arg::new("pubkey").required(true).help("base64 public key"))
                .arg(Arg::new("msg").required(true).help("Message to encrypt")),
        )
        .subcommand(
            Command::new("decrypt")
                .about("Decrypt a message")
                .arg(Arg::new("prikey").required(true).help("base64 private key"))
                .arg(Arg::new("msg").required(true).help("Message to decrypt")),
        )
        .get_matches();

    let mut rng = rand::thread_rng();

    match matches.subcommand() {
        Some(("genkey", _)) => {
            let (pr, pb) = cli_genkeys(&mut rng);
            println!("PRIVATE: {}", pr);
            println!("PUBLIC: {}", pb);
        }
        Some(("encrypt", args)) => {
            let enc = cli_encrypt(
                &mut rng,
                args.get_one::<String>("pubkey").unwrap(),
                args.get_one::<String>("msg").unwrap(),
            );
            println!("{}", enc);
        }
        Some(("decrypt", args)) => {
            let dec = cli_decrypt(
                &mut rng,
                args.get_one::<String>("prikey").unwrap(),
                args.get_one::<String>("msg").unwrap(),
            );
            println!("{}", dec);
        }
        _ => panic!(),
    }
}

fn cli_genkeys(rng: &mut impl Rng) -> (String, String) {
    let (pr, pb) = gen_keys(rng);
    (pr.base64(), pb.base64())
}

fn cli_encrypt(rng: &mut impl Rng, pubkey: &str, msg: &str) -> String {
    let pb = PublicKey::from_base64(pubkey);
    encrypt_message_and_encode(pb, msg, rng)
}

fn cli_decrypt(rng: &mut impl Rng, prikey: &str, msg: &str) -> String {
    let pr = PrivateKey::from_base64(prikey);
    decode_message_and_decrypt(pr, msg, rng)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::{cli_decrypt, cli_encrypt, cli_genkeys};

    #[test]
    fn full() {
        let text = "Hello, world!! :)";

        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..300 {
            let (pr, pb) = cli_genkeys(&mut gen);
            let enc = cli_encrypt(&mut gen, &pb, text);
            let dec = cli_decrypt(&mut gen, &pr, &enc);
            assert_eq!(dec, text);
        }
    }
}
