#![feature(iter_array_chunks)]
use clap::{Arg, Command};
use ecc::{gen_keys, PrivateKey, PublicKey};
use encoding_utils::{
    decode_message_and_decrypt, encrypt_message_and_encode, points_to_text, text_to_points,
};

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
            let (pr, pb) = gen_keys(&mut rng);
            println!("PRIVATE: {}", pr.base64());
            println!("PUBLIC: {}", pb.base64());
        }
        Some(("encrypt", args)) => {
            let pb = args.get_one::<String>("pubkey").unwrap();
            let pb = PublicKey::from_base64(pb);
            let msg = args.get_one::<String>("msg").unwrap();
            let enc = encrypt_message_and_encode(pb, msg, &mut rng);
            println!("{}", enc);
        }
        Some(("decrypt", args)) => {
            let pr = args.get_one::<String>("prikey").unwrap();
            let pr = PrivateKey::from_base64(pr);
            let msg = args.get_one::<String>("msg").unwrap();
            let dec = decode_message_and_decrypt(pr, msg, &mut rng);
            println!("{}", dec);
        }
        _ => panic!(),
    }
}
