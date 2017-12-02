#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
extern crate regex;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

use quote::ToTokens;

decl_derive!([Token, attributes(expr, skip)] => token_derive);

fn token_derive(s: synstructure::Structure) -> quote::Tokens {
    let exprs: Vec<_> = s.variants().iter().map(|v| check_regex(&v.ast())).collect();
    let ids: Vec<_> = s.variants().iter().map(|v| v.ast().ident).collect();
    let re_ids: Vec<_> = ids.iter().map(|i| syn::Ident::new(format!("RE_{}",i.as_ref()))).collect();
    let re_ids2 = re_ids.clone();
    let exprs2 = exprs.clone();
    let regexes_decl = quote!{
        lazy_static! {
            #(
                static ref #re_ids: ::relexer::regex::Regex = {
                    ::relexer::regex::Regex::new(#exprs).unwrap()
                };
            )*
        }
    };
    let make_tokens: Vec<_> = s.variants().iter().zip(exprs2.iter()).map(|(v,ex)| {
        v.construct(|f, i| {
            let mut ty_tokens = quote::Tokens::new();
            let ty = &f.ty;
            ty.to_tokens(&mut ty_tokens);
            let ty_str = ty_tokens.as_ref();
            let idx = i+1;
            quote!{
                match c.get(#idx).unwrap().as_str().parse::<#ty>() {
                    Ok(k) => k,
                    Err(_) => {
                        return (Err(::relexer::Error::InvalidToken{
                            parsed: c.get(#idx).unwrap().as_str().to_owned(),
                            regex: #ex,
                            ty: #ty_str,
                        }), input);
                    }
                }
            }
        })
    }).collect();
    let skip_match = s.each_variant(|v| {
        let skip = has_skip(&v.ast());
        quote!(#skip)
    });
    s.bound_impl("::relexer::Token", quote! {
        fn produce<'_input>(mut input: &'_input str) -> (::relexer::Result<Self>, &'_input str) {
            #regexes_decl
            #(
                if let Some(c) = #re_ids2.captures(input) {
                    let m = c.get(0).unwrap();
                    input = &input[m.end()..];
                    return (Ok(#make_tokens), input);
                }
             )*
            return (Err(::relexer::Error::InvalidInput{unparsed:input.to_owned()}), input);
        }
        fn skip(&self) -> bool {
            match *self {
                #skip_match
            }
        }
    })
}

fn has_skip(v: &synstructure::VariantAst) -> bool {
    for a in v.attrs {
        match a.value {
            syn::MetaItem::Word(ref id) => {
                if id == "skip" {
                    return true
                }
            },
            _ => {}
        }
    }
    return false;
}

fn check_regex(v: &synstructure::VariantAst) -> String {
    let mut expr = None;
    for a in v.attrs {
        match a.value {
            syn::MetaItem::NameValue(ref id, ref lit) => {
                match *lit {
                    syn::Lit::Str(ref l, _) => {
                        if id == "expr" {
                            expr = Some(l.as_str());
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
    let e = expr.expect(&format!("#[derive(Token)] requires all variants to have the `expr` attribute: `{}`", v.ident.as_ref()));
    let regex = regex::Regex::new(e).expect(&format!("#[derive(Token)] Invalid regex for variant `{}`",v.ident.as_ref()));
    let tuple_args = match *v.data {
        syn::VariantData::Struct(_) => panic!("#[derive(Token)] does not support struct variants"),
        syn::VariantData::Tuple(ref n) => n.len(),
        syn::VariantData::Unit => 0
    };
    if regex.captures_len()-1 != tuple_args {
        panic!("#[derive(Token)] Mismatch between number of capture groups and variant fields for variant `{}`",v.ident.as_ref());
    }
    format!("^{}",e)
}

