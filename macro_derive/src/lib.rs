extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(Check)]
pub fn macro_derive(input: TokenStream) -> TokenStream {
    impl_check_macro(input)
}
fn impl_check_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            panic!("expected a struct with named fields")
        }
    };
    let fields = fields.iter().map(|field| (&field.ident));
    TokenStream::from(quote! {
        impl Check for #struct_name{
            fn check(&self) -> Result<(), String> {
                #(
                    match (&self.#fields).ok(){
                        Ok(()) => {},
                        Err(e) => {
                            return Err(e)
                        }
                    }
                )*
                Ok(())
            }
        }
    })
}
