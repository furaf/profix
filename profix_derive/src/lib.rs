#![recursion_limit = "32768"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(FixSerialize, attributes(msg_type, id))]
pub fn fix_serialize(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_fix_serialize(ast);
    gen.parse().unwrap()
}

#[proc_macro_derive(FixDeserialize, attributes(msg_type, id))]
pub fn fix_deserialize(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_fix_deserialize(ast);
//    panic!("{}", gen);
    match gen.parse() {
        Ok(ts) => ts,
        Err(e) => panic!("{:?}: {:?}", e, gen),
    }
}

#[proc_macro_derive(FixDeserializeGroup, attributes(id))]
pub fn fix_deserialize_group(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_fix_deserialize_group(ast);
//    panic!("{}", gen);
    match gen.parse() {
        Ok(ts) => ts,
        Err(e) => panic!("{:?}: {:?}", e, gen),
    }
}

#[proc_macro_derive(FixParse, attributes(fix_value))]
pub fn fix_parse(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_fix_parse(ast);
    match gen.parse() {
        Ok(ts) => ts,
        Err(e) => panic!("{:?}: {:?}", e, gen),
    }
}

fn impl_fix_serialize(ast: syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let msg_type = find_attr("msg_type", &ast.attrs);

    if let syn::Body::Struct(syn::VariantData::Struct(fields)) = ast.body {
        let fields = find_fix_fields(&fields);

        let ids = fields.iter().map(|f| f.id.to_string());
        let idents = fields.iter().map(|f| f.ident.clone());
        let dummy_const = syn::Ident::new(format!("_IMPL_FIX_SERIALIZE_FOR_{}", name));

        quote! {
            #[allow(non_upper_case_globals)]
            const #dummy_const: () = {
                extern crate fix;
                impl fix::detail::FixSerializable for #name {
                    fn serialize_body_to_fix(&self) -> String {
                        format!(concat!("35=", #msg_type, "\x01", concat!(#(#ids, "={}\x01"),* )), #(self.#idents),*)
                    }
                }
            };
        }
    } else {
        panic!("#[derive(FixSerialize)] is only defined for structs")
    }
}


fn impl_fix_deserialize_group(ast: syn::DeriveInput) -> quote::Tokens {
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields)) => {
            impl_fix_deserialize_group_struct(ast.ident, ast.attrs, fields)
        }
        _ => panic!("#[derive(FixDeserializeGroup)] is only defined for structs"),
    }
}

fn impl_fix_deserialize(ast: syn::DeriveInput) -> quote::Tokens {
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(fields)) => {
            impl_fix_deserialize_struct(ast.ident, ast.attrs, fields)
        }
        syn::Body::Enum(variants) => impl_fix_deserialize_enum(ast.ident, variants),
        _ => panic!("#[derive(FixDeserialize)] is only defined for structs and enums"),
    }
}

fn impl_fix_parse(ast: syn::DeriveInput) -> quote::Tokens {
    match ast.body {
        syn::Body::Enum(variants) => impl_fix_parse_enum(ast.ident, variants),
        _ => panic!("#[derive(FixParse)] is only defined for enums"),
    }
}

fn impl_fix_parse_enum(name: syn::Ident, variants: Vec<syn::Variant>) -> quote::Tokens {
    let pairs: Vec<_> = variants
        .iter()
        .map(|variant| match variant.data {
            syn::VariantData::Unit => {
                let attr = find_attr("fix_value", &variant.attrs);
                (attr, variant.ident.clone())
            }
            _ => panic!("#[derive(FixParse)] Only unit variants are supported for enums."),
        })
        .collect();

    let names: Vec<_> = pairs.iter().map(|_| name.clone()).collect();
    let values: Vec<_> = pairs.iter().map(|p| p.0.as_bytes()).collect();
    let idents: Vec<_> = pairs.iter().map(|p| p.1.clone()).collect();
    let dummy_const = syn::Ident::new(format!("_IMPL_FIX_PARSE_FOR_{}", name));

    let tokens = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate fix;
            impl fix::FixParse for #name {
                fn parse(value: &[u8]) -> Result<Self, fix::ParseError> {
                    #(
                        if value == #values {
                            return Ok(#names::#idents);
                        }
                    )*

                    Err(concat!(stringify!(#name), " could not parse enum value"))
                }
            }
        };
    };
    tokens
}

struct ParserInternals {
    intros: Vec<quote::Tokens>,
    parses: Vec<quote::Tokens>,
    conses: Vec<quote::Tokens>,
}

fn generate_parser_internals(name: &syn::Ident, fields: &Vec<FixField>) -> ParserInternals {
    let mut intros = vec![];
    let mut parses = vec![];
    let mut conses = vec![];

    for field in fields.iter() {
        let out = &field.ident;
        let id = field.id;
        let err_multiple = format!("{} found multiple {}", name, out);
        match field.ty{
            syn::Ty::Path(_, ref path) if path.segments.last().unwrap().ident == "Vec" => {
                let err_input_end_before_checksum = format!("{} input ended before checksum", name);
                intros.push(quote! {
                    let mut #out = None;
                });
                parses.push(quote! {
                    #id => {
                        use fix::detail::FixDeserializableGroup as _FDG;

                        let _len: usize = fix::FixParse::parse(_field.value)?;
                        if _input.len() <= _field.length {
                            return Err(#err_input_end_before_checksum);
                        }
                        _input = &_input[_field.length..];
                        _checksum += _field.checksum;

                        if #out.is_none() {
                            let (_vec, _cont) = _FDG::deserialize_group_from_fix(_len, _input)?;
                            #out = Some(_vec);
                            _checksum += _cont.checksum;
                            _input = _cont.next_input;
                            _field = _cont.next_field;
                            continue;
                        } else {
                            return Err(#err_multiple);
                        }
                    },
                });
                conses.push(quote!{
                    #out: #out.unwrap_or(Vec::new())
                });
            },
            syn::Ty::Path(_, ref path) if path.segments.last().unwrap().ident == "Option" => {
                intros.push(quote! {
                    let mut #out = None;
                });
                parses.push(quote! {
                    #id => {
                        if #out.is_none() {
                            #out = Some(fix::FixParse::parse(_field.value)?);
                        } else {
                            return Err(#err_multiple);
                        }
                    },
                });
                conses.push(quote!{
                    #out: #out
                });
            },
            _ => {
                let err_missing = format!("{} missing {}", name, out);
                intros.push(quote! {
                    let mut #out = Err(#err_missing);
                });
                parses.push(quote! {
                    #id => {
                        if #out.is_err() {
                            #out = Ok(fix::FixParse::parse(_field.value)?);
                        } else {
                            return Err(#err_multiple);
                        }
                    },
                });
                conses.push(quote!{
                    #out: #out?
                });
            },
        }
    }

    ParserInternals {intros, parses, conses}
}


fn impl_fix_deserialize_group_struct(
    name: syn::Ident,
    attrs: Vec<syn::Attribute>,
    fields: Vec<syn::Field>,
) -> quote::Tokens {
    let fields = find_fix_fields(&fields);
    let ParserInternals { intros, parses, conses } = generate_parser_internals(&name, &fields);
    let parses_head = &parses[0];
    let parses_tail = &parses[1..];

    let dummy_const = syn::Ident::new(format!("_IMPL_FIX_DESERIALIZE_GROUP_FOR_{}", name));

    let err_input_end_before_checksum = format!("{} input ended before checksum", name);

    let tokens = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate fix;

            impl fix::detail::FixDeserializableGroup for #name {
                fn deserialize_group_from_fix(_expected_length: usize, _input_arg: &[u8])
                    -> Result<(Vec<Self>, fix::detail::ParserContinuation), fix::ParseError>
                {
                    let mut _input = _input_arg;
                    let mut _checksum = ::std::num::Wrapping(0u8);
                    let mut _out = Vec::new();
                    _out.reserve(_expected_length);

                    let mut _field = fix::detail::parse_fix_field(_input)?;
                    loop {
                        #( #intros )*

                        match _field.id {
                            #parses_head
                            _ => {
                                let cont = fix::detail::ParserContinuation {
                                    checksum: _checksum,
                                    next_input: _input,
                                    next_field: _field,
                                };
                                return Ok((_out, cont));
                            }
                        }

                        if _input.len() <= _field.length {
                            return Err(#err_input_end_before_checksum);
                        }
                        _input = &_input[_field.length..];
                        _checksum += _field.checksum;
                        _field = fix::detail::parse_fix_field(_input)?;

                        loop {
                            match _field.id {
                                #( #parses_tail )*
                                _ => break
                            }

                            if _input.len() <= _field.length {
                                return Err(#err_input_end_before_checksum);
                            }
                            _input = &_input[_field.length..];
                            _checksum += _field.checksum;
                            _field = fix::detail::parse_fix_field(_input)?;
                        }

                        _out.push(#name {
                            #( #conses ),*
                        });
                    }
                }
            }
        };
    };

    tokens
}


fn impl_fix_deserialize_struct(
    name: syn::Ident,
    attrs: Vec<syn::Attribute>,
    fields: Vec<syn::Field>,
) -> quote::Tokens {
    const CHECKSUM_ID: u64 = 10;

    let msg_type = find_attr("msg_type", &attrs);
    let msg_type_bytes = msg_type.as_bytes();

    let fields = find_fix_fields(&fields);
    let ParserInternals { intros, parses, conses } = generate_parser_internals(&name, &fields);

    let dummy_const = syn::Ident::new(format!("_IMPL_FIX_DESERIALIZE_FOR_{}", name));

    let err_invalid_checksum = format!("{} checksums do not match", name);
    let err_input_end_before_checksum = format!("{} input ended before checksum", name);
//    let err_input_after_checksum = format!("{} detected input after checksum", name);

    let tokens = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate fix;
            use std::num::Wrapping;

            impl fix::detail::FixMessageType for #name {
                const MSG_TYPE: &'static [u8] = &#msg_type_bytes;
            }

            impl fix::detail::FixDeserializable for #name {
                fn deserialize_from_fix(_msg: fix::detail::FixMessage) -> Result<Self, fix::ParseError> {
                    #( #intros )*

                    let mut _input = _msg.body;
                    let mut _checksum = _msg.header_checksum;
                    let mut _field = fix::detail::parse_fix_field(_input)?;
                    loop {
                        match _field.id {
                            #( #parses )*
                            #CHECKSUM_ID => {
                            // TODO: Fix this
                            /*
                                if input.len() != field.length {
                                    return Err(#err_input_after_checksum);
                                }
                            */
                                let _parsed_checksum: u8 = fix::FixParse::parse(_field.value)?;
                                if Wrapping(_parsed_checksum) != _checksum {
                                    return Err(#err_invalid_checksum);
                                }

                                return Ok(#name {
                                    #( #conses ),*
                                });
                            },
                            _ => {},
                        }

                        if _input.len() <= _field.length {
                            return Err(#err_input_end_before_checksum);
                        }
                        _checksum += _field.checksum;
                        _input = &_input[_field.length..];
                        _field = fix::detail::parse_fix_field(_input)?;
                    }
                }
            }
        };
    };

    tokens
}

fn impl_fix_deserialize_enum(name: syn::Ident, variants: Vec<syn::Variant>) -> quote::Tokens {
    let pairs: Vec<_> = variants
        .iter()
        .map(|variant| match variant.data {
            syn::VariantData::Tuple(ref fields) if fields.len() == 1 => {
                let field = &fields[0];
                match field.ty {
                    syn::Ty::Path(ref o, ref p) => {
                        let ty = syn::Ty::Path(o.clone(), p.clone());
                        (variant.ident.clone(), ty)
                    }
                    _ => panic!("Only paths are supported."),
                }
            }
            _ => panic!("Only one-field tuple variants are supported for enums."),
        })
        .collect();

    let names: Vec<_> = pairs.iter().map(|_| name.clone()).collect();
    let cases: Vec<_> = pairs.iter().map(|p| p.0.clone()).collect();
    let tys: Vec<_> = pairs.iter().map(|p| p.1.clone()).collect();
    let dummy_const = syn::Ident::new(format!("_IMPL_FIX_DESERIALIZE_FOR_{}", name));

    let tokens = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate fix;
            use fix::detail::FixMessageType;

            impl fix::detail::FixDeserializable for #name {
                fn deserialize_from_fix(msg: fix::detail::FixMessage) -> Result<Self, fix::ParseError> {
                    #(
                        if msg.msg_type == #tys::MSG_TYPE {
                            return Ok(#names::#cases(fix::detail::FixDeserializable::deserialize_from_fix(msg)?));
                        }
                    )*

                    Err(concat!(stringify!(#name), " unknown FIX message"))
                }
            }
        };
    };
    tokens
}

struct FixField {
    id: u64,
    ident: syn::Ident,
    ty: syn::Ty,
}

fn find_fix_fields(fields: &[syn::Field]) -> Vec<FixField> {
    fields
        .iter()
        .map(|field| {
            let id = find_attr("id", &field.attrs);
            let id: u64 = match id.parse() {
                Ok(id) => id,
                Err(e) => panic!("Could not parse ID as u64: {} {}", id, e),
            };
            FixField {
                id,
                ident: field.ident.clone().unwrap(),
                ty: field.ty.clone(),
            }
        })
        .collect()
}

fn find_attr(name: &str, attrs: &[syn::Attribute]) -> String {
    let mut result = None;
    for attr in attrs {
        if let syn::AttrStyle::Outer = attr.style {
            if let syn::MetaItem::NameValue(ident, lit) = attr.value.clone() {
                if syn::Ident::new(name) == ident {
                    if let syn::Lit::Str(value, _style) = lit {
                        if result.is_none() {
                            result = Some(value);
                        } else {
                            panic!("{} supplied twice", name);
                        }
                    } else {
                        panic!("{} must be a string", name);
                    }
                }
            }
        }
    }

    match result {
        Some(x) => x,
        None => panic!("{} not found", name),
    }
}