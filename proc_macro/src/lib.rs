use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, ItemFn, Token};

struct StaticMapperIdent(Option<Token![mut]>, Ident);

impl Parse for StaticMapperIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![mut]) {
            input
                .parse::<Token![mut]>()
                .and_then(|m| input.parse::<Ident>().map(|ident| Self(Some(m), ident)))
        } else {
            input.parse::<Ident>().map(|ident| Self(None, ident))
        }
    }
}

#[proc_macro_attribute]
pub fn map_statics(attr: TokenStream, item: TokenStream) -> TokenStream {
    let statics =
        parse_macro_input!(attr with Punctuated::<StaticMapperIdent, Token![,]>::parse_terminated);
    let mut func = parse_macro_input!(item as ItemFn);
    statics
        .into_iter()
        .rev()
        .for_each(|StaticMapperIdent(mutable, ident)| {
            let mapped_ident = Ident::new(ident.to_string().to_lowercase().as_str(), ident.span());
            let block = func.block.as_ref();
            let wrapper_func: Ident = match mutable {
                Some(_) => parse_quote!(with_borrow_mut),
                None => parse_quote!(with_borrow),
            };
            func.block = parse_quote! {
                {
                    #ident.#wrapper_func(|#mapped_ident| #block)
                }
            };
        });
    func.into_token_stream().into()
}
