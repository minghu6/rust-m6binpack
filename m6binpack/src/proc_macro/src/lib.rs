use proc_macro::TokenStream;
use syn::parse::{
    Parse, ParseStream
};
use syn:: {
    Ident, LitInt, Token, Type, parse_macro_input
};
use quote::{format_ident, quote};


extern crate proc_macro;
#[macro_use]
extern crate alloc;

/// <VarName: VarType: bitlen>
struct BitVar(Ident, Type, LitInt);
struct BinUnpackItem(Vec<BitVar>, Ident);
struct BinUnPackUsize {
    vars_vec: Vec<BinUnpackItem>,
}

impl Parse for BinUnPackUsize {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut vars_vec = vec![];

        while !input.is_empty() {
            let mut vars = vec![];
            while !input.peek(Token![=]) {
                input.parse::<Token![<]>()?;
                let varname: Ident = input.parse()?;
                input.parse::<Token![:]>()?;

                let vartype: Type = input.parse()?;
                input.parse::<Token![:]>()?;

                let bitlen: LitInt = input.parse()?;
                input.parse::<Token![>]>()?;

                vars.push(BitVar(varname, vartype, bitlen));
            }

            input.parse::<Token![=]>()?;
            let target = input.parse()?;
            input.parse::<Token![;]>()?;

            vars_vec.push(
                BinUnpackItem(vars, target)
            )
        }

        Ok(Self {
            vars_vec
        })
    }
}

#[proc_macro]
pub fn unpack(input: TokenStream) -> TokenStream {
    let BinUnPackUsize {
        vars_vec
    } = parse_macro_input!(input as BinUnPackUsize);

    let mut token_stream = quote! {};
    token_stream.extend(quote! {
        use m6binpack::{
            extract_bits
        };
        let mut _lensum: usize;
    });

    for BinUnpackItem(vars, target) in vars_vec {
        token_stream.extend(quote! {
            _lensum = 0;
        });
    for BitVar(varname, vartype, bitlen) in vars {
        let tmpvar = format_ident!("_tmp_{}", varname);

        token_stream.extend(quote! {
            let #tmpvar: usize = extract_bits(#target, _lensum, #bitlen);
        });

        match vartype {
            // >=1 true, == 0 false
            Type::Path(type_path) if type_path.path.is_ident("bool") => {
                token_stream.extend(quote! {
                    let #varname = #tmpvar >= 1;
                });
            },
            _ => {
                token_stream.extend(quote! {
                    let #varname = #tmpvar as #vartype;
                });
            }
        }

        token_stream.extend(quote! {
            _lensum += #bitlen;
        });
    }}

    TokenStream::from(token_stream)
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
