mod derive_enum;
mod derive_struct;

use derive_enum::derive_enum;
use derive_struct::derive_struct;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};

#[proc_macro_derive(Compress)]
pub fn derive_compress(item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as Item);

  match item {
    Item::Enum(item) => return derive_enum(&item),
    Item::Struct(item) => return derive_struct(&item),
    _ => panic!("Only `enums` and `structs` can derive Compress"),
  }
}
