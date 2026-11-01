# KiCad Rust API

Allows you to develop add-ons and tools that interact with a running instance
of [KiCad](https://kicad.org).  This library is built against the official
KiCad IPC API.

For more information about the IPC API, please see the [KiCad developer documentation](https://dev-docs.kicad.org).

```rust
use kicad::KiCad;

fn main() {
    let k = KiCad::new(KiCadConnectionConfig {
        client_name: String::from("version-example"),
        ..Default::default()
    })
    .expect("KiCad not running!");

    println!("Connected to KiCad {}", k.get_version().unwrap());
}
```

## Documentation

There is no documentation yet, sorry.  The first priority of the team is completing the KiCad side
of the new IPC API, and enabling a smooth transition for users of the existing SWIG Python
bindings.  If you are interested in contributing to the Rust bindings, please get in touch!

## Contributing

At the moment, these bindings are being developed in parallel with the IPC API itself, and
development is being coordinated by the KiCad team (main point of contact: Jon Evans / @craftyjon).
Expect rapid change and instability during this development period, and please do not send merge
requests without discussing your idea for changes with the team ahead of time.

Once the initial stable API is released (planned for KiCad 9.0 in February 2025), this Cargo
package will also have its first stable release and be considered ready for general use.  Until
that time, please consider this a development preview.

## Developing

Install `pre-commit` using `pip` or your preferred method.  Then run `pre-commit install` to add
the pre-commit hooks to your working directory.

The library can be built with a simple `cargo build`, and examples can be run from `cargo` as well,
e.g. `cargo run --example version`.
