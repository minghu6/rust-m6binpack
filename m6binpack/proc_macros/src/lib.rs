use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Brace,
    ExprBlock, Ident, LitInt, Token, Type,
};

extern crate proc_macro;
#[macro_use]
extern crate alloc;

////////////////////////////////////////////////////////////////////////////////
//// Procedural Macros

/// LSB parsing
///
/// ```
/// use proc_macros::unpack;
/// use m6binpack::Unpack;
///
/// let cause: u32 = 0x8000_000A;
///
/// unpack! {
///     <cause_num: usize: 31><is_async: bool: 1> = cause;
///     <B0: usize: 12><_: 12><B1: u8: 8> = cause;
/// };
///
/// assert_eq!(cause_num, 0x0A);
/// assert_eq!(is_async, true);
/// assert_eq!(B0, 0x0A);
/// assert_eq!(B1, 8 << 4);
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
/// ```
/// use proc_macros::pack;
/// use m6binpack::Pack;
///
/// let cause_num = 0x8000_000A;
/// let is_async = 1;
///
/// pack! {
///     cause0: u32 = <cause_num: 31><is_async: 1>;
/// };
///
/// assert_eq!(cause0, 0x8000_000A);
///
/// pack! {
///     cause0 |= <4: 12><0: 12><0x02: 8>
/// }
///
/// assert_eq!(cause0, 0x8200_000E, "{cause0:0X}");
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

enum Val {
    ExprBlock(ExprBlock),
    LitInt(LitInt),
    Ident(Ident),
}

/// <VarName: VarType: bitlen>
struct UnpackVar(Option<(Ident, Type)>, Val);

struct UnpackItem(Vec<UnpackVar>, Ident);

struct UnPackItemList(Vec<UnpackItem>);

struct PackItemList(Vec<PackItem>);

enum PackItem {
    Init(Ident, Type, Vec<PackVar>),
    AddAssign(Ident, Vec<PackVar>),
}

struct PackVar(Val, Val);

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl Parse for Val {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Brace) {
            Val::ExprBlock(input.parse()?)
        }
        else if input.peek(Ident) {
            Val::Ident(input.parse()?)
        }
        else {
            Val::LitInt(input.parse()?)
        })
    }
}

impl ToTokens for Val {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let totokens: &dyn ToTokens = match self {
            Val::ExprBlock(expr_block) => expr_block,
            Val::LitInt(lit_int) => lit_int,
            Val::Ident(ident) => ident,
        };

        totokens.to_tokens(tokens);
    }
}


/// <_: intlit>
impl Parse for PackVar {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;

        let val = input.parse::<Val>()?;
        input.parse::<Token![:]>()?;
        let bitlen = input.parse::<Val>()?;

        input.parse::<Token![>]>()?;

        Ok(Self(val, bitlen))
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

                while !input.peek(Token![;]) && !input.is_empty() {
                    vars.push(input.parse::<PackVar>()?);
                }

                item_list.push(PackItem::Init(name, ty, vars));
            } else {
                input.parse::<Token![|=]>()?;

                let mut vars = vec![];

                while !input.peek(Token![;]) && !input.is_empty() {
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
                } else {
                    // eprintln!("varname_: None");
                    input.parse::<Token![_]>()?;
                    None
                };

                input.parse::<Token![:]>()?;
                let bitlen = input.parse::<Val>()?;

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

    let mut ts = quote! {
        #[allow(unused_braces)]
    };

    let func_name;

    if lsb {
        func_name = Ident::new("extract", Span::call_site());
    } else {
        func_name = Ident::new("extract_msb", Span::call_site());
    }

    for UnpackItem(vars, target) in item_list {
        ts.extend(quote! {
            let mut _lensum = 0usize;
        });

        for UnpackVar(var, bitlen) in vars {
            if let Some((varname, vartype)) = var {
                let tmpvar = format_ident!("_tmp_{}", varname);

                ts.extend(quote! {
                    let _bitlen = #bitlen;

                    #[allow(non_snake_case)]
                    let #tmpvar = Unpack::#func_name(
                        &#target, _lensum + 1..=_lensum + _bitlen
                    );
                });

                match vartype {
                    // !=0 true, ==0 false
                    Type::Path(type_path) if type_path.path.is_ident("bool") => {
                        ts.extend(quote! {
                            #[allow(mut_unused)]
                            #[allow(non_snake_case)]
                            let mut #varname = #tmpvar != 0;
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
                _lensum += _bitlen;
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
        func_name = Ident::new("insert", Span::call_site());
    } else {
        func_name = Ident::new("insert_msb", Span::call_site());
    }

    for PackVar(val, bitlen) in vars {
        ts.extend(quote! {
            let _val = #val;
            let _bitlen = #bitlen;

            Pack::#func_name(
                &mut #target,
                _val,
                _lensum + 1..=_lensum + _bitlen
            );

            _lensum += _bitlen;
        });
    }

    ts
}
