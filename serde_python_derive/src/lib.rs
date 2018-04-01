extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(Python)]
pub fn serde_python(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    impl_serde_python(&ast).into()
}

fn impl_serde_python(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        impl serde_python::cpython::ToPyObject for #name {
            type ObjectType = serde_python::cpython::PyObject;

            fn to_py_object(&self, py: serde_python::cpython::Python) -> Self::ObjectType {
                use serde_python::serde::Serialize;
                self.serialize(serde_python::PyObjectSerializer::new(py)).unwrap()
            }
        }
    }
}
