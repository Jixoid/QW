/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Generics, Ident};


fn is_mapped(attrs: &[Attribute]) -> bool {
  attrs.iter().any(|attr| {
    if attr.path().is_ident("serial") {
      let mut found = false;
      let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("map") || meta.path.is_ident("mapped") {
          found = true;
        }
        Ok(())
      });
      found
    } else {
      false
    }
  })
}

// Encode
#[proc_macro_derive(Encode, attributes(serial))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let generics = add_trait_bounds(ast.generics);

  if is_mapped(&ast.attrs) {
    return encode_mapped(&ast.ident, &generics);
  }

  // Select TypeKind
  match ast.data {
    Data::Enum(ref e) => encode_enum(&ast.ident, &generics, e),
    Data::Struct(ref e) => encode_struct(&ast.ident, &generics, e),
    _ => panic!("Encode makrosu sadece Enum ve Struct'lar için kullanılabilir!"),
  }
}

#[proc_macro_derive(MappedEncode)]
pub fn derive_mapped_encode(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let generics = add_trait_bounds(ast.generics);
  encode_mapped(&ast.ident, &generics)
}

fn encode_mapped(name: &Ident, generics: &Generics) -> TokenStream {
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let expanded = quote! {
    impl #impl_generics ::qwc_serial::Encoder for #name #ty_generics #where_clause {
      fn encode(&self, ctx: &mut ::qwc_serial::EncodeCtx) -> Result<(), ::qwc_serial::EncodeErr> {
        let id = ctx.map_encode(self)?;
        <u32 as ::qwc_serial::Encoder>::encode(&id, ctx)?;
        Ok(())
      }
    }
  };
  TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
  for param in &mut generics.params {
    if let syn::GenericParam::Type(ref mut type_param) = *param {
      type_param.bounds.push(syn::parse_quote!(::qwc_serial::Encoder));
    }
  }
  generics
}

fn encode_enum(name: &Ident, generics: &Generics, e: &DataEnum) -> TokenStream {
  let match_arms = e.variants.iter().enumerate().map(|(index, variant)| {
    let variant_name = &variant.ident;
    let index_u16 = index as u16;

    match &variant.fields {
      syn::Fields::Unit => {
        quote! {
          #name::#variant_name => {
            <u16 as ::qwc_serial::Encoder>::encode(&#index_u16, ctx)?;
          }
        }
      }

      syn::Fields::Unnamed(fields) => {
        let bindings: Vec<_> = (0..fields.unnamed.len()).map(|i| format_ident!("f{}", i)).collect();
        let types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();

        quote! {
          #name::#variant_name( #(#bindings),* ) => {
            <u16 as ::qwc_serial::Encoder>::encode(&#index_u16, ctx)?;
            #(
              <#types as ::qwc_serial::Encoder>::encode(#bindings, ctx)?;
            )*
          }
        }
      }

      syn::Fields::Named(fields) => {
        let idents: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
        let types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

        quote! {
          #name::#variant_name { #(#idents),* } => {
            <u16 as ::qwc_serial::Encoder>::encode(&#index_u16, ctx)?;
            #(
              <#types as ::qwc_serial::Encoder>::encode(#idents, ctx)?;
            )*
          }
        }
      }
    }
  });

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let expanded = quote! {
    impl #impl_generics ::qwc_serial::Encoder for #name #ty_generics #where_clause {
      fn encode(&self, ctx: &mut ::qwc_serial::EncodeCtx) -> Result<(), ::qwc_serial::EncodeErr> {
        match self {
          #(#match_arms)*
        }
        Ok(())
      }
    }
  };

  TokenStream::from(expanded)
}

fn encode_struct(name: &Ident, generics: &Generics, e: &DataStruct) -> TokenStream {
  let fields = e.fields.iter().enumerate().map(|(idx, it)| {
    let ty = &it.ty;

    if let Some(ident) = &it.ident {
      quote! {
        <#ty as ::qwc_serial::Encoder>::encode(&self.#ident, ctx)?;
      }
    } else {
      let idx = syn::Index::from(idx);
      quote! {
        <#ty as ::qwc_serial::Encoder>::encode(&self.#idx, ctx)?;
      }
    }
  });

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let expanded = quote! {
    impl #impl_generics ::qwc_serial::Encoder for #name #ty_generics #where_clause {
      fn encode(&self, ctx: &mut ::qwc_serial::EncodeCtx) -> Result<(), ::qwc_serial::EncodeErr> {
        #(#fields)*
        Ok(())
      }
    }
  };

  TokenStream::from(expanded)
}


// Decode
#[proc_macro_derive(Decode, attributes(serial))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let generics = add_decode_trait_bounds(ast.generics);

  if is_mapped(&ast.attrs) {
    return decode_mapped(&ast.ident, &generics);
  }

  // Select TypeKind
  match ast.data {
    Data::Enum(ref e) => decode_enum(&ast.ident, &generics, e),
    Data::Struct(ref e) => decode_struct(&ast.ident, &generics, e),
    _ => panic!("Decode makrosu sadece Enum ve Struct'lar için kullanılabilir!"),
  }
}

#[proc_macro_derive(MappedDecode)]
pub fn derive_mapped_decode(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let generics = add_decode_trait_bounds(ast.generics);
  decode_mapped(&ast.ident, &generics)
}

fn decode_mapped(name: &Ident, generics: &Generics) -> TokenStream {
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let expanded = quote! {
    impl #impl_generics ::qwc_serial::Decoder for #name #ty_generics #where_clause {
      fn decode(ctx: &mut ::qwc_serial::DecodeCtx) -> Result<Self, ::qwc_serial::DecodeErr> {
        let id = <u32 as ::qwc_serial::Decoder>::decode(ctx)?;
        ctx.map_decode::<Self>(id)
      }
    }
  };
  TokenStream::from(expanded)
}

fn add_decode_trait_bounds(mut generics: Generics) -> Generics {
  for param in &mut generics.params {
    if let syn::GenericParam::Type(ref mut type_param) = *param {
      type_param.bounds.push(syn::parse_quote!(::qwc_serial::Decoder));
    }
  }
  generics
}

fn decode_enum(name: &Ident, generics: &Generics, e: &DataEnum) -> TokenStream {
  let match_arms = e.variants.iter().enumerate().map(|(index, variant)| {
    let variant_name = &variant.ident;
    let index_u16 = index as u16;

    match &variant.fields {
      syn::Fields::Unit => {
        quote! {
          #index_u16 => Ok(Self::#variant_name),
        }
      }

      syn::Fields::Unnamed(fields) => {
        let recurse = fields.unnamed.iter().map(|f| {
          let ty = &f.ty;
          quote! { <#ty as ::qwc_serial::Decoder>::decode(ctx)? }
        });

        quote! {
          #index_u16 => Ok(Self::#variant_name( #(#recurse),* )),
        }
      }

      syn::Fields::Named(fields) => {
        let recurse = fields.named.iter().map(|f| {
          let ident = &f.ident;
          let ty = &f.ty;
          quote! { #ident: <#ty as ::qwc_serial::Decoder>::decode(ctx)? }
        });

        quote! {
          #index_u16 => Ok(Self::#variant_name { #(#recurse),* }),
        }
      }
    }
  });

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let expanded = quote! {
    impl #impl_generics ::qwc_serial::Decoder for #name #ty_generics #where_clause {
      fn decode(ctx: &mut ::qwc_serial::DecodeCtx) -> Result<Self, ::qwc_serial::DecodeErr> {
        let tag = <u16 as ::qwc_serial::Decoder>::decode(ctx)?;

        match tag {
          #(#match_arms)*

          _ => Err(::qwc_serial::DecodeErr::UnknownTag),
        }
      }
    }
  };

  TokenStream::from(expanded)
}

fn decode_struct(name: &Ident, generics: &Generics, e: &DataStruct) -> TokenStream {
  let construct = match &e.fields {
    syn::Fields::Named(fields) => {
      let recurse = fields.named.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! { #ident: <#ty as ::qwc_serial::Decoder>::decode(ctx)? }
      });
      quote! { Ok(Self { #(#recurse),* }) }
    }

    syn::Fields::Unnamed(fields) => {
      let recurse = fields.unnamed.iter().map(|f| {
        let ty = &f.ty;
        quote! { <#ty as ::qwc_serial::Decoder>::decode(ctx)? }
      });
      quote! { Ok(Self( #(#recurse),* )) }
    }

    syn::Fields::Unit => {
      quote! { Ok(Self) }
    }
  };

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let expanded = quote! {
    impl #impl_generics ::qwc_serial::Decoder for #name #ty_generics #where_clause {
      fn decode(ctx: &mut ::qwc_serial::DecodeCtx) -> Result<Self, ::qwc_serial::DecodeErr> {
        #construct
      }
    }
  };

  TokenStream::from(expanded)
}
