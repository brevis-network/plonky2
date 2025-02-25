# Plonky2 & more
[![Discord](https://img.shields.io/discord/743511677072572486?logo=discord)](https://discord.gg/QZKRUpqCJ6)

This repository was originally for Plonky2, a SNARK implementation based on techniques from PLONK and FRI. It has since expanded to include tools such as Starky, a highly performant STARK implementation.


## Documentation

For more details about the Plonky2 argument system, see this [writeup](plonky2/plonky2.pdf).

Polymer Labs has written up a helpful tutorial [here](https://polymerlabs.medium.com/a-tutorial-on-writing-zk-proofs-with-plonky2-part-i-be5812f6b798)!


## Examples

A good starting point for how to use Plonky2 for simple applications is the included examples:

* [`factorial`](plonky2/examples/factorial.rs): Proving knowledge of 100!
* [`fibonacci`](plonky2/examples/fibonacci.rs): Proving knowledge of the hundredth Fibonacci number
* [`range_check`](plonky2/examples/range_check.rs): Proving that a field element is in a given range
* [`square_root`](plonky2/examples/square_root.rs): Proving knowledge of the square root of a given field element

To run an example, use

```sh
cargo run --example <example_name>
```


## Building

Plonky2 requires a recent nightly toolchain, although we plan to transition to stable in the future.

To use a nightly toolchain for Plonky2 by default, you can run
```
rustup override set nightly
```
in the Plonky2 directory.


## Running

To see recursion performance, one can run this bench, which generates a chain of three recursion proofs:

```sh
RUSTFLAGS=-Ctarget-cpu=native cargo run --release --example bench_recursion -- -vv
```

## Jemalloc

Plonky2 prefers the [Jemalloc](http://jemalloc.net) memory allocator due to its superior performance. To use it, include `jemallocator = "0.5.0"` in your `Cargo.toml` and add the following lines
to your `main.rs`:

```rust
use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

Jemalloc is known to cause crashes when a binary compiled for x86 is run on an Apple silicon-based Mac under [Rosetta 2](https://support.apple.com/en-us/HT211861). If you are experiencing crashes on your Apple silicon Mac, run `rustc --print target-libdir`. The output should contain `aarch64-apple-darwin`. If the output contains `x86_64-apple-darwin`, then you are running the Rust toolchain for x86; we recommend switching to the native ARM version.

## Contributing guidelines

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## Licenses

All crates of this monorepo are licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


## Security

This code has not yet been audited, and should not be used in any production systems.

While Plonky2 is configurable, its defaults generally target 100 bits of security. The default FRI configuration targets 100 bits of *conjectured* security based on the conjecture in [ethSTARK](https://eprint.iacr.org/2021/582).

Plonky2's default hash function is Poseidon, configured with 8 full rounds, 22 partial rounds, a width of 12 field elements (each ~64 bits), and an S-box of `x^7`. [BBLP22](https://tosc.iacr.org/index.php/ToSC/article/view/9850) suggests that this configuration may have around 95 bits of security, falling a bit short of our 100 bit target.


## Links

#### Actively maintained

- [Polygon Zero's zkEVM](https://github.com/0xPolygonZero/zk_evm), an efficient Type 1 zkEVM built on top of Starky and plonky2

#### No longer maintained

- [System Zero](https://github.com/0xPolygonZero/system-zero), a zkVM built on top of Starky
- [Waksman](https://github.com/0xPolygonZero/plonky2-waksman), Plonky2 gadgets for permutation checking using Waksman networks
- [Insertion](https://github.com/0xPolygonZero/plonky2-insertion), Plonky2 gadgets for insertion into a list
- [u32](https://github.com/0xPolygonZero/plonky2-u32), Plonky2 gadgets for u32 arithmetic
- [ECDSA](https://github.com/0xPolygonZero/plonky2-ecdsa), Plonky2 gadgets for the ECDSA algorithm


The Extension Field is defined be $\gamma = g_0 + g_1 \cdot \sqrt{W}$ where $g_0, g_1 \in \text{GF}(p)$, ${W}$ = 7;

Assume there are 3 column a, b and c in the column which <T> is a Goldilocks Field in trace. the gamma will be splited to 2 base fields $g_0$, $g_1$ here.

The combination result:

$\text{RLC}_i = \gamma \cdot a_i + \gamma^2 \cdot b_i + \gamma^3 \cdot c_i$

Expanded into base field components:

$\text{RLC}_i^{(0)} = a_i g_0 + b_i(g_0^2 + 7g_1^2) + c_i(g_0^3 + 21g_0g_1^2)$ 

$\text{RLC}_i^{(1)} = a_i g_1 + b_i(2g_0g_1) + c_i(3g_0^2g_1 + 7g_1^3)$


---------------------------
And there is another proposal.

The Extension Field is defined be $\gamma = g_0 + g_1 \cdot \sqrt{W}$ where $g_0, g_1 \in \text{GF}(p)$, ${W}$ = 7 in Goldilocks QuadraticExtension

Assume there are 3 column a, b and c in the column which <T> is a Goldilocks Field in trace. the gamma will be splited to 2 base fields $g_0$, $g_1$ here.

The combination result:

$\text{RLC}_i = \gamma \cdot a_i + \gamma^2 \cdot b_i + \gamma^3 \cdot c_i$

Expanded into base field components:

$\text{RLC}_i^{(0)} = a_i g_0 + b_i(g_0^2 + 7g_1^2) + c_i(g_0^3 + 21g_0g_1^2)$ 

$\text{RLC}_i^{(1)} = a_i g_1 + b_i(2g_0g_1) + c_i(3g_0^2g_1 + 7g_1^3)$


$\text{RLC}_i^{(0)} = a_i g_0 + b_i(g_0^2 + 7g_1^2) + c_i(g_0^3 + 21g_0g_1^2) + d_i(g_0^4 + 42g_0^2g_1^2 + 49g_1^4)$

$\text{RLC}_i^{(1)} = a_i g_1 + 2b_i g_0 g_1 + c_i(3g_0^2 g_1 + 7g_1^3) + d_i(4g_0^3 g_1 + 28g_0 g_1^3)$.
