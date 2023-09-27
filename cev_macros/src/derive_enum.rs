use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Fields, GenericParam, ItemEnum};

pub fn derive_enum(item: &ItemEnum) -> TokenStream {
  let ident = &item.ident;
  let generics = &item.generics;

  let generic_names = generics
    .params
    .iter()
    .map(|param| match param {
      GenericParam::Lifetime(ty) => ty.lifetime.to_token_stream(),
      GenericParam::Type(ty) => ty.ident.to_token_stream(),
      GenericParam::Const(ty) => ty.ident.to_token_stream(),
    })
    .collect::<Vec<_>>();

  let variants = &item.variants;

  let mut compress = Vec::new();
  let mut uncompress = Vec::new();
  let mut size = Vec::new();
  let mut read_size = Vec::new();

  variants.iter().enumerate().for_each(|(i, variant)| {
    let i = i as u8;
    let variant_ident = &variant.ident;
    match &variant.fields {
      Fields::Unit => {
        compress.push(quote! {
          #ident::#variant_ident => {
            unsafe { *ptr = #i; }
            offset += 1;
          }
        });
        uncompress.push(quote! {
          #i => {
            return #ident::#variant_ident;
          }
        });
        size.push(quote! {
          #ident::#variant_ident => {
            return 1;
          }
        });
        read_size.push(quote! {
          #i => {
            return 1;
          }
        });
      }
      Fields::Unnamed(fields) => {
        let unnamed = &fields.unnamed;

        let mut names = Vec::new();
        let mut compress_actions = Vec::new();
        let mut uncompress_actions = Vec::new();
        let mut size_actions = Vec::new();
        let mut read_size_actions = Vec::new();
        unnamed.iter().enumerate().for_each(|(i, field)| {
          let ident = format_ident!("x{}", i);
          names.push(ident.clone());

          compress_actions.push(quote! {
            #ident.compress(unsafe { ptr.add(offset) });
            offset += #ident.size();
          });

          let ty = &field.ty;
          uncompress_actions.push(quote! {
            let #ident = #ty::uncompress(unsafe { ptr.add(offset) });
            offset += #ident.size();
          });

          size_actions.push(quote! {
            #ident.size()
          });

          read_size_actions.push(quote! {
            offset += #ty::read_size(unsafe { ptr.add(offset) });
          });
        });

        compress.push(quote! {
          #ident::#variant_ident (#(#names),*) => {
            unsafe { *ptr = #i; }
            offset += 1;
            #(#compress_actions)*
          }
        });
        uncompress.push(quote! {
          #i => {
            #(#uncompress_actions)*
            return #ident::#variant_ident (#(#names),*);
          }
        });
        size.push(quote! {
          #ident::#variant_ident (#(#names),*) => {
            return #(#size_actions)+* + 1;
          }
        });
        read_size.push(quote! {
          #i => {
            return offset;
          }
        });
      }
      Fields::Named(fields) => {
        let named = &fields.named;

        let mut names = Vec::new();
        let mut compress_actions = Vec::new();
        let mut uncompress_actions = Vec::new();
        let mut size_actions = Vec::new();
        let mut read_size_actions = Vec::new();
        named.iter().for_each(|field| {
          let ident = field
            .ident
            .as_ref()
            .expect("Since it's a named field, it should have an ident");
          names.push(ident.clone());

          compress_actions.push(quote! {
            #ident.compress(unsafe { ptr.add(offset) });
            offset += #ident.size();
          });

          let ty = &field.ty;
          uncompress_actions.push(quote! {
            let #ident = #ty::uncompress(unsafe { ptr.add(offset) });
            offset += #ident.size();
          });

          size_actions.push(quote! {
            #ident.size()
          });

          read_size_actions.push(quote! {
            offset += #ty::read_size(unsafe { ptr.add(offset) });
          });
        });

        compress.push(quote! {
          #ident::#variant_ident {#(#names),*} => {
            unsafe { *ptr = #i; }
            offset += 1;
            #(#compress_actions)*
          }
        });
        uncompress.push(quote! {
          #i => {
            #(#uncompress_actions)*
            return #ident::#variant_ident {#(#names),*};
          }
        });
        size.push(quote! {
          #ident::#variant_ident {#(#names),*} => {
            return #(#size_actions)+* + 1;
          }
        });
        read_size.push(quote! {
          #i => {
            return offset;
          }
        });
      }
    }
  });

  return quote! {
    impl #generics cev::Compress for #ident < #(#generic_names),* > {
      fn compress(&self, ptr: *mut u8) {
        let mut offset = 0;
        match self {
          #(#compress)*
        }
      }

      fn uncompress(ptr: *const u8) -> Self {
        let variant = unsafe { *ptr };
        let mut offset = 1;

        match variant {
          #(#uncompress)*
          _ => unreachable!("Invalid variant, we should have matched them all with the macro")
        }
      }

      fn size(&self) -> usize {
        match self {
          #(#size)*
        }
      }

      fn read_size(ptr: *const u8) -> usize {
        let variant = unsafe { *ptr };
        let mut offset = 1;

        match variant {
          #(#read_size)*
          _ => unreachable!("Invalid variant, we should have matched them all with the macro")
        }
      }
    }
  }
  .into();
}
