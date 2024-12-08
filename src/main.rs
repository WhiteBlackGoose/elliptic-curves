#![feature(iter_array_chunks)]
#![feature(generic_const_exprs)]
#![feature(cursor_remaining)]
use algebra::CommutativeOp;
use base_traits::{FromRandom, Natural, RW};
use clap::{Arg, Command};
use ecc::{gen_keys, PrivateKey, PublicKey};
use encoding_utils::{
    decode_message_and_decrypt, encrypt_message_and_encode, points_to_text, text_to_points,
};
use mod_field::{ModField, ModFieldCfg};
use points_group::{Point, PointCfg};
use rand::Rng;

mod algebra;
mod base_traits;
mod ecc;
mod encoding_utils;
mod mod_field;
mod points_group;

fn main() {
    let cfg_field = ModFieldCfg {
        rem: 0x0014_4C3B_27FFu64,
        // 0x1FFF_FFFF_FFFF_FFFF
    };
    let cfg_group = PointCfg {
        g: Point::new_unsafe(
            ModField::new(2500, &cfg_field),
            ModField::new(125001, &cfg_field),
        ),
        a: ModField::new(100, &cfg_field),
        b: ModField::new(1, &cfg_field),
        cf: cfg_field,
    };

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
            let (pr, pb) = cli_genkeys::<u128, u64>(&mut rng, &cfg_group);
            println!("PRIVATE: {}", pr);
            println!("PUBLIC: {}", pb);
        }
        Some(("encrypt", args)) => {
            let enc = cli_encrypt(
                &mut rng,
                args.get_one::<String>("pubkey").unwrap(),
                args.get_one::<String>("msg").unwrap(),
                &cfg_group,
            );
            println!("{}", enc);
        }
        Some(("decrypt", args)) => {
            let dec = cli_decrypt(
                args.get_one::<String>("prikey").unwrap(),
                args.get_one::<String>("msg").unwrap(),
                &cfg_group,
            );
            println!("{}", dec);
        }
        _ => panic!(),
    }
}

fn cli_genkeys<IP: Natural + FromRandom<()> + RW, I: Natural + RW>(
    rng: &mut impl Rng,
    cfg: &PointCfg<ModField<I>>,
) -> (String, String) {
    let (pr, pb) = gen_keys::<_, IP, Point<ModField<I>>>(rng, cfg);
    (pr.base64(), pb.base64())
}

fn cli_encrypt<I: Natural + RW + FromRandom<()>>(
    rng: &mut impl Rng,
    pubkey: &str,
    msg: &str,
    cfg: &PointCfg<ModField<I>>,
) -> String
where
    [(); ModField::<I>::LEN - 1]:,
    [(); Point::<ModField<I>>::LEN]:,
{
    let pb = PublicKey::from_base64(pubkey);
    encrypt_message_and_encode::<ModField<I>, I>(pb, msg, rng, cfg)
}

fn cli_decrypt<I: Natural + RW + FromRandom<()>>(
    prikey: &str,
    msg_base64: &str,
    cfg: &PointCfg<ModField<I>>,
) -> String
where
    [(); Point::<ModField<I>>::LEN]:,
    [(); ModField::<I>::LEN]:,
{
    let pr = PrivateKey::from_base64(prikey);
    decode_message_and_decrypt::<I, ModField<I>>(pr, msg_base64, cfg)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::{
        cli_decrypt, cli_encrypt, cli_genkeys,
        mod_field::{ModField, ModFieldCfg},
        points_group::{Point, PointCfg},
    };

    #[test]
    fn full() {
        let cfg_field = ModFieldCfg {
            rem: 0x0014_4C3B_27FFu64,
            // 0x1FFF_FFFF_FFFF_FFFF
        };
        let cfg_group = PointCfg {
            g: Point::new_unsafe(
                ModField::new(2500, &cfg_field),
                ModField::new(125001, &cfg_field),
            ),
            a: ModField::new(100, &cfg_field),
            b: ModField::new(1, &cfg_field),
            cf: cfg_field,
        };

        let text = "Hello, world!! :)";
        println!("{:?}", text.bytes());

        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..300 {
            let (pr, pb) = cli_genkeys::<u128, u64>(&mut gen, &cfg_group);
            let enc = cli_encrypt(&mut gen, &pb, text, &cfg_group);
            let dec = cli_decrypt(&pr, &enc, &cfg_group);
            assert_eq!(dec, text);
        }
    }
}
