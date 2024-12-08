#![feature(iter_array_chunks)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![feature(cursor_remaining)]
use base_traits::{FromRandom, Natural, RW};
use clap::{Arg, Command};
use ecc::{gen_keys, PrivateKey, PublicKey};
use encoding_utils::{decode_message_and_decrypt, encrypt_message_and_encode};
use mod_field::{ModField, ModFieldCfg};
use points_group::{Point, PointCfg};
use primitive_types::U256;
use rand::Rng;

mod algebra;
mod base_traits;
mod ecc;
mod encoding_utils;
mod mod_field;
mod points_group;

type DatatypeScalar = U256;
type DatatypeShort = U256;

// https://en.bitcoin.it/wiki/Secp256k1
fn secp256k1() -> PointCfg<ModField<DatatypeShort>> {
    let cfg_field = ModFieldCfg {
        rem: U256::from_big_endian(&[
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFC, 0x2F,
        ]),
    };
    let gx = U256::from_big_endian(&[
        0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87, 0x0B,
        0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B, 0x16, 0xF8,
        0x17, 0x98,
    ]);
    let gy = U256::from_big_endian(&[
        0x48, 0x3A, 0xDA, 0x77, 0x26, 0xA3, 0xC4, 0x65, 0x5D, 0xA4, 0xFB, 0xFC, 0x0E, 0x11, 0x08,
        0xA8, 0xFD, 0x17, 0xB4, 0x48, 0xA6, 0x85, 0x54, 0x19, 0x9C, 0x47, 0xD0, 0x8F, 0xFB, 0x10,
        0xD4, 0xB8,
    ]);
    let cfg_group = PointCfg {
        g: Point::new_unsafe(ModField::new(gx, &cfg_field), ModField::new(gy, &cfg_field)),
        a: ModField::new(U256::from(0), &cfg_field),
        b: ModField::new(U256::from(7), &cfg_field),
        cf: cfg_field,
    };

    assert_eq!(size_of::<DatatypeScalar>(), size_of::<U256>());
    assert_eq!(size_of::<DatatypeShort>(), size_of::<U256>());

    cfg_group
}

fn main() {
    let cfg_group = secp256k1();

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
            let (pr, pb) = cli_genkeys::<DatatypeScalar, DatatypeShort>(&mut rng, &cfg_group);
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
            let dec = cli_decrypt::<DatatypeScalar, DatatypeShort>(
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

fn cli_decrypt<IP: Natural + FromRandom<()> + RW, I: Natural + RW + FromRandom<()>>(
    prikey: &str,
    msg_base64: &str,
    cfg: &PointCfg<ModField<I>>,
) -> String
where
    [(); Point::<ModField<I>>::LEN]:,
    [(); ModField::<I>::LEN]:,
{
    let pr = PrivateKey::<IP>::from_base64(prikey);
    decode_message_and_decrypt::<IP, ModField<I>>(pr, msg_base64, cfg)
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

        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let (pr, pb) = cli_genkeys::<u128, u64>(&mut gen, &cfg_group);
            let enc = cli_encrypt(&mut gen, &pb, text, &cfg_group);
            let dec = cli_decrypt::<u128, u64>(&pr, &enc, &cfg_group);
            assert_eq!(dec, text);
        }
    }
}
