# My naive implementation of ECC

ECC - elliptic curves cryptography. This is just my hello world project to learn a bit about ECC.

It works in general case with any length of key as long as the types implement a list of traits. `algebra.rs` contains all relevant traits from group theory, and `base_traits.rs` also some technical traits, that's how keys and elements of the group can be substituted.

As a default, I use `secp256k1` with the bitcoin's parameters, because why not.

Usage: see `--help`.

# Sources

- [post 1](https://hackernoon.com/what-is-the-math-behind-elliptic-curve-cryptography-f61b25253da3)
- [post 2](https://www.rareskills.io/post/elliptic-curve-addition)
- [post 3](https://andrea.corbellini.name/2015/05/17/elliptic-curve-cryptography-a-gentle-introduction/)
- [post 4](https://andrea.corbellini.name/2023/01/02/ec-encryption/)
