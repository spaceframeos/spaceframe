<h1 align="center">
    <img src="https://avatars.githubusercontent.com/u/80397804?s=100&v=4">
    <p>Spaceframe</p>
</h1>

<h3 align="center">A Swiss ecological and confidential blockchain</h3>
<h4 align="center"><i>:warning: For education purposes only. This is by no means a complete implementation and it is by no means secure!</i></h4>

## Getting started

To test the blockchain, you must install [cargo](https://www.rust-lang.org/tools/install) first.

After cloning the repository, build the project in release mode :

```
cargo build --release
```

Then you need to create a plot to be able to find proofs :

```
cargo r --release --bin spaceframe-node -- init -k <choose a number>
```

Choose a `k` value between these values: 17, 19, 20, 22, 23. Higher is better but takes more time to generate. Best value for testing is **k = 23**.

After the plot has been generated, you can play with the blockchain in local (for the moment) with the `demo` command :

```
cargo r --release --bin spaceframe-node -- demo -k <enter the same number as before>
```
