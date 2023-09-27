use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Fields, GenericParam, ItemStruct};

pub fn derive_struct(item: &ItemStruct) -> TokenStream {
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

  let generic_names = if generic_names.is_empty() {
    quote! {}
  } else {
    quote! { < #(#generic_names),* > }
  };

  match &item.fields {
    Fields::Unit => {
      return quote! {
        impl #generics cev::Compress for #ident #generic_names {
          fn compress(&self, ptr: *mut u8) {
            return;
          }

          fn uncompress(ptr: *const u8) -> Self {
            return #ident;
          }

          fn size(&self) -> usize {
            return 0;
          }

          fn read_size(ptr: *const u8) -> usize {
            return 0;
          }
        }
      }
      .into()
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

      return quote! {
        impl #generics cev::Compress for #ident #generic_names {
          fn compress(&self, ptr: *mut u8) {
            let mut offset = 0;
            let #ident (#(#names),*) = &self;
            #(#compress_actions)*
          }

          fn uncompress(ptr: *const u8) -> Self {
            let mut offset = 0;
            #(#uncompress_actions)*
            return #ident(#(#names),*);
          }

          fn size(&self) -> usize {
            let #ident (#(#names),*) = &self;
            return #(#size_actions)+*;
          }

          fn read_size(ptr: *const u8) -> usize {
            let mut offset = 0;
            #(#read_size_actions)*
            return offset;
          }
        }
      }
      .into();
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
          self.#ident.compress(unsafe { ptr.add(offset) });
          offset += self.#ident.size();
        });

        let ty = &field.ty;
        uncompress_actions.push(quote! {
          let #ident = #ty::uncompress(unsafe { ptr.add(offset) });
          offset += #ident.size();
        });

        size_actions.push(quote! {
          self.#ident.size()
        });

        read_size_actions.push(quote! {
          offset += #ty::read_size(unsafe { ptr.add(offset) });
        });
      });

      return quote! {
        impl #generics cev::Compress for #ident #generic_names {
          fn compress(&self, ptr: *mut u8) {
            let mut offset = 0;
            #(#compress_actions)*
          }

          fn uncompress(ptr: *const u8) -> Self {
            let mut offset = 0;
            #(#uncompress_actions)*
            return #ident {#(#names),*};
          }

          fn size(&self) -> usize {
            return #(#size_actions)+*;
          }

          fn read_size(ptr: *const u8) -> usize {
            let mut offset = 0;
            #(#read_size_actions)*
            return offset;
          }
        }
      }
      .into();
    }
  }
}
