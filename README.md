# serde_python

A helper library that allows you to use serde-enabled structures seamlessly with cpython

## Getting Started

```
#[macro_use]
extern crate serde_python;

#[derive(Python, Serialize)]
struct MyStruct {
  // ...
}
```

## Limitations / buyer-beware

Right now, simple enums serialize to strings, more complex enums aren't supported.

Advice on how to correctly handle this is welcome.

## Coming Soon

Support for FromPyObject (deserialize direction)

## Running the tests

```
cargo test -- --nocapture`
```

The output must currently be manually audited by a human.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details

