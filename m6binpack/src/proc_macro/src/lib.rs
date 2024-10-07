use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, Token, Type,
};

extern crate proc_macro;
#[macro_use]
extern crate alloc;

////////////////////////////////////////////////////////////////////////////////
//// Procedural Macros

/// LSB parsing
///
/// ```no_run
/// let cause: usize = 0x8000_0000_0000_000A;
///
/// unpack! {
///     <cause_num: usize: 63><is_async: bool: 1> = cause;
///     <B0: usize: 12><B1: u8: 4><B2: u8: 8> = cause;
/// };
/// ```
#[proc_macro]
pub fn unpack(input: TokenStream) -> TokenStream {
    unpack_(input, true)
}

/// MSB parsing
#[proc_macro]
pub fn unpack_msb(input: TokenStream) -> TokenStream {
    unpack_(input, false)
}

/// LSB packing
///
/// ```no_run
/// let cause_num = 0x8000_0000_0000_000A;
/// let is_async = 1;
///
/// pack! {
///     cause0: u64 = <cause_num: 63><is_async: 1>;
///     cause0 += <4: 12><3: 4><0: 40><2: 8>;
/// };
/// ```
#[proc_macro]
pub fn pack(input: TokenStream) -> TokenStream {
    pack_(input, true)
}

#[proc_macro]
pub fn pack_msb(input: TokenStream) -> TokenStream {
    pack_(input, false)
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

/// <VarName: VarType: bitlen>
struct UnpackVar(Option<(Ident, Type)>, LitInt);

struct UnpackItem(Vec<UnpackVar>, Ident);

struct UnPackItemList(Vec<UnpackItem>);

struct PackItemList(Vec<PackItem>);

enum PackItem {
    Init(Ident, Type, Vec<PackVar>),
    AddAssign(Ident, Vec<PackVar>),
}

enum PackVar {
    Var(Ident, LitInt),
    Lit(LitInt, LitInt),
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

/// <_: intlit>
impl Parse for PackVar {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek2(Ident) {
            input.parse::<Token![<]>()?;

            let name = input.parse::<Ident>()?;
            let bitlen = input.parse::<LitInt>()?;

            input.parse::<Token![>]>()?;

            Self::Var(name, bitlen)
        } else {
            input.parse::<Token![<]>()?;

            let lit = input.parse::<LitInt>()?;
            let bitlen = input.parse::<LitInt>()?;

            input.parse::<Token![>]>()?;

            Self::Lit(lit, bitlen)
        })
    }
}

impl Parse for PackItemList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut item_list = vec![];

        while !input.is_empty() {
            let name = input.parse::<Ident>()?;

            if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                let ty = input.parse::<Type>()?;
                input.parse::<Token![=]>()?;

                let mut vars = vec![];

                while !input.peek(Token![;]) {
                    vars.push(input.parse::<PackVar>()?);
                }

                item_list.push(PackItem::Init(name, ty, vars));
            } else {
                input.parse::<Token![+=]>()?;

                let mut vars = vec![];

                while !input.peek(Token![;]) {
                    vars.push(input.parse::<PackVar>()?);
                }

                item_list.push(PackItem::AddAssign(name, vars));
            }

            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }

        Ok(Self(item_list))
    }
}

impl Parse for UnPackItemList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut item_list = vec![];

        while !input.is_empty() {
            let mut item = vec![];

            while !input.peek(Token![=]) {
                input.parse::<Token![<]>()?;

                let var = if let Some(varname_) = input.parse::<Option<Ident>>()? {
                    // eprintln!("varname_: {}", varname_.to_string());
                    input.parse::<Token![:]>()?;

                    let vartype: Type = input.parse()?;

                    Some((varname_, vartype))
                }
                else {
                    // eprintln!("varname_: None");
                    input.parse::<Token![_]>()?;
                    None
                };

                input.parse::<Token![:]>()?;
                let bitlen = input.parse::<LitInt>()?;

                input.parse::<Token![>]>()?;

                item.push(UnpackVar(var, bitlen));
            }

            input.parse::<Token![=]>()?;
            let target = input.parse()?;

            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }

            item_list.push(UnpackItem(item, target))
        }

        Ok(Self(item_list))
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

fn unpack_(input: TokenStream, lsb: bool) -> TokenStream {
    let UnPackItemList(item_list) = parse_macro_input!(input as UnPackItemList);

    let mut ts = quote! {};

    for UnpackItem(vars, target) in item_list {
        ts.extend(quote! {
            let mut _lensum = 0usize;
        });

        for UnpackVar(var, bitlen) in vars {
            if let Some((varname, vartype)) = var {
                let tmpvar = format_ident!("_tmp_{}", varname);

                if lsb {
                    ts.extend(quote! {
                        #[allow(non_snake_case)]
                        let #tmpvar = m6binpack::Unpack::extract(
                            &#target, _lensum + 1..=_lensum + #bitlen
                        );
                    });
                } else {
                    ts.extend(quote! {
                        #[allow(non_snake_case)]
                        let #tmpvar = m6binpack::Unpack::extract_msb(
                            &#target, _lensum + 1..=_lensum + #bitlen
                        );
                    });
                }

                match vartype {
                    // >=1 true, == 0 false
                    Type::Path(type_path) if type_path.path.is_ident("bool") => {
                        ts.extend(quote! {
                            #[allow(mut_unused)]
                            #[allow(non_snake_case)]
                            let mut #varname = #tmpvar >= 1;
                        });
                    }
                    _ => {
                        ts.extend(quote! {
                            #[allow(mut_unused)]
                            #[allow(non_snake_case)]
                            let mut #varname = #tmpvar as #vartype;
                        });
                    }
                }
            }

            ts.extend(quote! {
                _lensum += #bitlen;
            });
        }
    }

    TokenStream::from(ts)
}

fn pack_(input: TokenStream, lsb: bool) -> TokenStream {
    let PackItemList(item_list) = parse_macro_input!(input as PackItemList);

    let mut ts = quote! {};

    for item in item_list {
        ts.extend(match item {
            PackItem::Init(target, ty, vars) => {
                let mut ts = quote! {
                    #[allow(mut_unused)]
                    #[allow(non_snake_case)]
                    let mut #target: #ty = 0;
                };

                ts.extend(pack_item_addassign(target, vars, lsb));

                ts
            }
            PackItem::AddAssign(target, vars) => pack_item_addassign(target, vars, lsb),
        })
    }

    TokenStream::from(ts)
}

fn pack_item_addassign(target: Ident, vars: Vec<PackVar>, lsb: bool) -> proc_macro2::TokenStream {
    let mut ts = quote! {
        let mut _lensum = 0usize;
    };

    let func_name;

    if lsb {
        func_name = Ident::new("insert_msb", Span::call_site());
    } else {
        func_name = Ident::new("insert", Span::call_site());
    }

    for var in vars {
        match var {
            PackVar::Var(name, bitlen) => {
                ts.extend(quote! {
                    m6binpack::Pack::#func_name(
                        &mut #target,
                        #name,
                        _lensum..=_lensum + #bitlen - 1
                    );
                    _lensum += #bitlen;
                });
            }
            PackVar::Lit(name, bitlen) => {
                ts.extend(quote! {
                    m6binpack::Pack::#func_name(
                        &mut #target,
                        #name,
                         _lensum..=_lensum + #bitlen - 1
                    );
                    _lensum += #bitlen;
                });
            }
        }
    }

    ts
}
