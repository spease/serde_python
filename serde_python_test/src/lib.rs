#[macro_use]
extern crate serde_python;
extern crate serde;
#[macro_use]
extern crate serde_derive;


#[derive(Clone,Python,Serialize)]
enum E {
    One,
    Two
}

#[derive(Clone,Python,Serialize)]
struct A {
    foo: String,
    bar: u64,
    baz: E,
    boop: E,
    chaz: C,
    to_be: bool,
    unit: ()
}
#[derive(Clone,Python,Serialize)]
struct C(u8, u8);
#[test]
fn serialize() {
    use serde_python::cpython::ObjectProtocol;
    let a = A {
        foo: "whee".to_string(),
        bar: 1337,
        baz: E::Two,
        boop: E::One,
        chaz: C(7,12),
        to_be: true,
        unit: ()
    };
    let gil = serde_python::cpython::Python::acquire_gil();
    let py = gil.python();
    let pprint = py.import("pprint").unwrap();
    let inspect = py.import("inspect").unwrap();
    pprint.get(py, "pprint").unwrap().call(py, (inspect.get(py, "getmembers").unwrap().call(py, (a.clone(),), None).unwrap(),) ,None).unwrap();
    pprint.get(py, "pprint").unwrap().call(py, (inspect.get(py, "getmembers").unwrap().call(py, (a,), None).unwrap(),) ,None).unwrap();
}
