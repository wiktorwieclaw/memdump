use darling::{ast::Data, FromDeriveInput};
use quote::quote;
use syn::{DeriveInput, Ident, Type, TypePtr};

#[derive(darling::FromDeriveInput)]
struct InputReceiver {
    ident: syn::Ident,
    data: Data<(), FieldReceiver>,
}

#[derive(darling::FromField)]
#[darling(attributes(memdump))]
struct FieldReceiver {
    ident: Option<Ident>,
    ty: Type,
    array: Option<ArrayReceiver>,
    c_string: Option<()>,
}

#[derive(darling::FromMeta)]
struct ArrayReceiver {
    len: Ident,
}

#[proc_macro_derive(Dump, attributes(memdump))]
pub fn derive_dump(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(tokens as DeriveInput);
    let receiver = InputReceiver::from_derive_input(&ast).unwrap();
    let ident = receiver.ident;
    let fields = receiver.data.take_struct().unwrap();
    let field_writes = fields.iter().map(|field| {
        let ident = &field.ident;
        let array = &field.array;
        let c_string = &field.c_string;

        if let Some(array) = array {
            assert!(matches!(
                field.ty,
                Type::Ptr(TypePtr {
                    mutability: None,
                    ..
                })
            ));

            let len = &array.len;

            quote! {
                for i in 0..self.#len {
                    let v = unsafe { self.#ident.offset(i as isize).read() };
                    nwritten += v.dump(&mut buf[nwritten..])
                }
            }
        } else if let Some(_c_string) = c_string {
            let Type::Ptr(TypePtr {
                mutability: None,
                elem: ty,
                ..
            }) = &field.ty
            else {
                panic!();
            };

            let Type::Path(path) = ty.as_ref() else {
                panic!();
            };

            if !path.path.is_ident("c_char") {
                panic!();
            }

            quote! {
                let mut i = 0;
                loop {
                    let c = unsafe { self.#ident.offset(i).read() };
                    i += 1;
                    nwritten += memdump::Dump::dump(&c, &mut buf[nwritten..]);
                    if c as u8 == b'\0' {
                        break;
                    }
                }
            }
        } else {
            quote! {
                nwritten += memdump::Dump::dump(&self.#ident, &mut buf[nwritten..]);
            }
        }
    });

    quote! {
        impl Dump for #ident {
            fn dump(&self, buf: &mut [u8]) -> usize {
                let mut nwritten = 0;
                #(#field_writes)*
                nwritten
            }
        }
    }
    .into()
}

#[proc_macro_derive(FromDump, attributes(memdump))]

pub fn derive_from_dump(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(tokens as DeriveInput);
    let receiver = InputReceiver::from_derive_input(&ast).unwrap();
    let ident = receiver.ident;
    let fields = receiver.data.take_struct().unwrap();
    let field_reads = fields.iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        let array = &field.array;
        let c_string = &field.c_string;

        if let Some(array) = array {
            let Type::Ptr(TypePtr {
                mutability: None,
                elem: ty,
                ..
            }) = ty
            else {
                panic!();
            };

            let len = &array.len;
            (
                quote! {
                    let #ident = buf[nread..].as_ptr() as *const _;
                    for _ in 0..#len {
                        let (_, n) = <#ty as memdump::FromDump>::from_dump(&buf[nread..]);
                        nread += n;
                    }
                },
                ident,
            )
        } else if let Some(_c_string) = c_string {
            let Type::Ptr(TypePtr {
                mutability: None,
                elem: ty,
                ..
            }) = &field.ty else {
                panic!();
            };
            
            let Type::Path(path) = ty.as_ref() else { 
                panic!();
            };
            
            if !path.path.is_ident("c_char") {
                panic!();
            }
            
            (
                quote! {
                    let #ident = buf[nread..].as_ptr() as *const c_char;
                    loop {
                        let c = buf[nread];
                        nread += 1;
                        if c == b'\0' {
                            break;
                        }
                    }
                },
                ident,
            )
        } else {
            (
                quote! {
                    let (#ident, n) = <#ty as memdump::FromDump>::from_dump(&buf[nread..]);
                    nread += n;
                },
                ident,
            )
        }
    });

    let (field_reads, fields): (Vec<_>, Vec<_>) = field_reads.unzip();

    quote! {
        impl FromDump for #ident {
            fn from_dump(buf: &[u8]) -> (Self, usize) {
                let mut nread = 0;
                #(#field_reads)*;

                (Self { #(#fields),* }, nread)
            }
        }
    }
    .into()
}
