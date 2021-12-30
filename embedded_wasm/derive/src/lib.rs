use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::{fs::File, io::Write};
use syn::{spanned::Spanned, Error, FnArg, ImplItem, ImplItemMethod, ItemImpl, ReturnType};

#[proc_macro]
pub fn derive_ffi_handler(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let result = derive_ffi_handler_inner(stream.into())
        .unwrap_or_else(Error::into_compile_error)
        .into();

    let mut fs = File::create("target/derive_ffi_handler.rs").unwrap();
    writeln!(&mut fs, "{}", result).unwrap();
    drop(fs);
    result
}

fn derive_ffi_handler_inner(stream: TokenStream) -> Result<TokenStream, Error> {
    let item_impl: ItemImpl = syn::parse2(stream)?;
    let (impl_generics, _type_generics, _where_clause) = item_impl.generics.split_for_impl();

    let mut functions = item_impl
        .items
        .into_iter()
        .map(Function::new)
        .collect::<Result<Vec<_>, _>>()?;

    let unhandled = match functions
        .iter()
        .position(|f| f.has_unhandled_attribute())
        .map(|i| functions.remove(i))
    {
        Some(fun) => {
            let mut fun = fun.item;
            fun.attrs.clear();
            Some(fun)
        }
        None => None,
    };

    let item_ty = item_impl.self_ty;

    let function_name_match = functions
        .iter()
        .map(|f| f.match_statement())
        .collect::<Result<Vec<_>, _>>()?;
    let fn_defs = functions.iter().map(|f| &f.item).collect::<Vec<_>>();

    export_ffi(&functions, "target/embedded_wasm_ffi.rs")
        .expect("Could not export target/embedded_wasm_ffi.rs");

    Ok(quote! {
        impl #impl_generics embedded_wasm::FfiHandler for #item_ty {
            fn handle(&mut self, process: &mut embedded_wasm::Process, function_name: &str, args: embedded_wasm::Vec<embedded_wasm::Dynamic>) {
                match function_name {
                    #(#function_name_match,)*
                    _ => self.unhandled(function_name, args),
                }
            }

            #unhandled
        }

        impl #impl_generics #item_ty {
            #(#fn_defs)*
        }
    })
}

fn export_ffi(functions: &[Function], out: &str) -> std::io::Result<()> {
    let mut file = File::create(out)?;
    writeln!(&mut file, "extern \"C\" {{")?;
    for function in functions {
        let name = function.item.sig.ident.to_string();
        write!(&mut file, "    pub fn {}(", name)?;
        for (idx, arg) in function
            .item
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(t) = &arg {
                    Some(t)
                } else {
                    None
                }
            })
            .enumerate()
        {
            if idx != 0 {
                write!(&mut file, ", ")?;
            }
            write!(
                &mut file,
                "{}: {}",
                arg.pat.to_token_stream(),
                arg.ty.to_token_stream()
            )?;
        }
        write!(&mut file, ")")?;

        if let ReturnType::Type(_, ty) = &function.item.sig.output {
            write!(&mut file, " -> {}", ty.to_token_stream())?;
        }
        writeln!(&mut file, ";")?;
    }
    writeln!(&mut file, "}}")?;

    Ok(())
}

struct Function {
    item: ImplItemMethod,
}

impl Function {
    pub fn new(item: ImplItem) -> Result<Self, Error> {
        match item {
            ImplItem::Method(item) => Ok(Self { item }),
            item => Err(Error::new(item.span(), "Only methods are supported")),
        }
    }

    pub fn match_statement(&self) -> Result<TokenStream, Error> {
        let mut error = None;
        let name = &self.item.sig.ident;
        let output = match &self.item.sig.output {
            ReturnType::Type(_, ty) => Some(ty),
            ReturnType::Default => None,
        };
        let mut stream = TokenStream::new();
        stream.extend([
            lit_str(name, name.span()),
            punct_joined('='),
            punct_joined('>'),
            group(Delimiter::Brace, |_stream| {
                for (idx, arg) in self.item.sig.inputs.iter().skip(1).enumerate() {
                    let name = format!("_{}", idx);
                    let ty = match arg {
                        FnArg::Typed(t) => &t.ty,
                        _ => panic!("Invalid type arg"),
                    };
                    let ty_name = ty.into_token_stream().to_string();
                    if let Err(e) = validate_ty_name(&ty_name) {
                        error = Some(Error::new(ty.span(), e));
                        break;
                    }
                    _stream.extend([
                        ident("let"),
                        ident(&name),
                        punct('='),
                        ident("args"),
                        punct('.'),
                        ident("get"),
                        group(Delimiter::Parenthesis, |stream| {
                            stream.extend([lit_usize(idx)])
                        }),
                        punct('.'),
                        ident("map"),
                        group(Delimiter::Parenthesis, |stream| {
                            let name = format!("as_{}", &ty_name);
                            stream.extend([
                                punct('|'),
                                ident("a"),
                                punct('|'),
                                ident("a"),
                                punct('.'),
                                ident_spanned(&name, ty.span()),
                                group(Delimiter::Parenthesis, |_| {}),
                            ])
                        }),
                        punct(';'),
                    ]);
                }
                let count = self.item.sig.inputs.len() - 1;
                let mut call_fn = {
                    let mut stream = TokenStream::new();
                    if output.is_some() {
                        stream.extend([ident("let"), ident("result"), punct('=')]);
                    }
                    stream.extend([
                        ident("self"),
                        punct('.'),
                        ident_spanned(&name.to_string(), name.span()),
                        group(Delimiter::Parenthesis, |args| {
                            for n in 0..count {
                                if n != 0 {
                                    args.extend([punct(',')]);
                                }
                                args.extend([ident(&format!("_{}", n))]);
                            }
                        }),
                        punct(';'),
                    ]);
                    if output.is_some() {
                        stream.extend([
                            ident("process"),
                            punct('.'),
                            ident("stack_push"),
                            group(Delimiter::Parenthesis, |stream| {
                                stream.extend([ident("result")])
                            }),
                            punct(';'),
                        ]);
                    }
                    stream.extend([ident("return"), punct(';')]);
                    stream
                };
                for n in 0..count {
                    call_fn = {
                        let mut stream = TokenStream::new();
                        stream.extend([
                            ident("if"),
                            ident("let"),
                            ident("Some"),
                            group(Delimiter::Parenthesis, |stream| {
                                stream.extend([ident(&format!("_{}", n))])
                            }),
                            punct('='),
                            ident(&format!("_{}", n)),
                            group(Delimiter::Brace, |stream| *stream = call_fn),
                        ]);
                        stream
                    };
                }
                _stream.extend(call_fn);
                _stream.extend([
                    ident("self"),
                    punct('.'),
                    ident("unhandled"),
                    group(Delimiter::Parenthesis, |inner| {
                        inner.extend([ident("function_name"), punct(','), ident("args")])
                    }),
                    punct(';'),
                ])
            }),
            // punct(','),
        ]);

        if let Some(error) = error {
            Err(error)
        } else {
            Ok(stream)
        }
    }
    fn has_unhandled_attribute(&self) -> bool {
        for attr in self.item.attrs.iter() {
            if attr.path.to_token_stream().to_string() == "unhandled" {
                return true;
            }
        }
        false
    }
}

fn lit_str(n: impl ToString, span: Span) -> TokenTree {
    let mut lit = Literal::string(&n.to_string());
    lit.set_span(span);
    lit.into()
}

fn lit_usize(val: usize) -> TokenTree {
    Literal::usize_unsuffixed(val).into()
}

fn punct_joined(c: char) -> TokenTree {
    Punct::new(c, Spacing::Joint).into()
}

fn punct(c: char) -> TokenTree {
    Punct::new(c, Spacing::Alone).into()
}

fn ident(i: &str) -> TokenTree {
    Ident::new(i, Span::call_site()).into()
}

fn ident_spanned(i: &str, span: Span) -> TokenTree {
    let mut ident = Ident::new(i, Span::call_site());
    ident.set_span(span);
    ident.into()
}

fn group(delimiter: Delimiter, group: impl FnOnce(&mut TokenStream)) -> TokenTree {
    let mut stream = TokenStream::new();
    group(&mut stream);

    Group::new(delimiter, stream).into()
}

fn validate_ty_name(name: &str) -> Result<(), String> {
    match name {
        "i32" | "i64" | "f32" | "f64" => Ok(()),
        x => Err(format!(
            "Invalid type {:?}, only i32, i64, f32 or f64 supported",
            x
        )),
    }
}
