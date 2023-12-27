use anyhow::{bail, Error};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Meta;

use crate::field::{set_bool, set_option, tag_attr, word_attr, Label};

#[derive(Clone)]
pub enum Kind {
    Optional,
    AlwaysEncode,
    Repeated,
}

impl From<Label> for Kind {
    fn from(value: Label) -> Self {
        match value {
            Label::Optional => Kind::Optional,
            Label::Repeated => Kind::Repeated,
        }
    }
}

#[derive(Clone)]
pub struct Field {
    pub kind: Kind,
    pub tag: u32,
}

impl Field {
    pub fn new(attrs: &[Meta], inferred_tag: Option<u32>) -> Result<Option<Field>, Error> {
        let mut message = false;
        let mut label = None;
        let mut tag = None;
        let mut boxed = false;

        let mut unknown_attrs = Vec::new();

        for attr in attrs {
            if word_attr("message", attr) {
                set_bool(&mut message, "duplicate message attribute")?;
            } else if word_attr("boxed", attr) {
                set_bool(&mut boxed, "duplicate boxed attribute")?;
            } else if let Some(t) = tag_attr(attr)? {
                set_option(&mut tag, t, "duplicate tag attributes")?;
            } else if let Some(l) = Label::from_attr(attr) {
                set_option(&mut label, l, "duplicate label attributes")?;
            } else {
                unknown_attrs.push(attr);
            }
        }

        if !message {
            return Ok(None);
        }

        match unknown_attrs.len() {
            0 => (),
            1 => bail!(
                "unknown attribute for message field: {:?}",
                unknown_attrs[0]
            ),
            _ => bail!("unknown attributes for message field: {:?}", unknown_attrs),
        }

        let tag = match tag.or(inferred_tag) {
            Some(tag) => tag,
            None => bail!("message field is missing a tag attribute"),
        };

        Ok(Some(Field {
            kind: label.unwrap_or(Label::Optional).into(),
            tag,
        }))
    }

    pub fn new_oneof(attrs: &[Meta]) -> Result<Option<Field>, Error> {
        if let Some(mut field) = Field::new(attrs, None)? {
            if let Some(attr) = attrs.iter().find(|attr| Label::from_attr(attr).is_some()) {
                bail!(
                    "invalid attribute for oneof field: {}",
                    attr.path().into_token_stream()
                );
            }
            field.kind = Kind::AlwaysEncode;
            Ok(Some(field))
        } else {
            Ok(None)
        }
    }

    pub fn encode(&self, ident: TokenStream) -> TokenStream {
        let tag = self.tag;
        match self.kind {
            Kind::Optional => quote! {
                if let Some(msg) = &#ident {
                    ::bilrost::encoding::message::encode(#tag, msg, buf, tw);
                }
            },
            Kind::AlwaysEncode => quote! {
                ::bilrost::encoding::message::encode(#tag, &#ident, buf, tw);
            },
            Kind::Repeated => quote! {
                for msg in &#ident {
                    ::bilrost::encoding::message::encode(#tag, msg, buf, tw);
                }
            },
        }
    }

    pub fn merge(&self, ident: TokenStream) -> TokenStream {
        match self.kind {
            Kind::Optional => quote! {
                ::bilrost::encoding::message::merge(wire_type,
                                                 #ident.get_or_insert_with(::core::default::Default::default),
                                                 buf,
                                                 ctx)
            },
            Kind::AlwaysEncode => quote! {
                ::bilrost::encoding::message::merge(wire_type, #ident, buf, ctx)
            },
            Kind::Repeated => quote! {
                ::bilrost::encoding::message::merge_repeated(wire_type, #ident, buf, ctx)
            },
        }
    }

    pub fn encoded_len(&self, ident: TokenStream) -> TokenStream {
        let tag = self.tag;
        match self.kind {
            Kind::Optional => quote! {
                #ident.as_ref().map_or(0, |msg| ::bilrost::encoding::message::encoded_len(#tag, msg, tm))
            },
            Kind::AlwaysEncode => quote! {
                ::bilrost::encoding::message::encoded_len(#tag, &#ident, tm)
            },
            Kind::Repeated => quote! {
                ::bilrost::encoding::message::encoded_len_repeated(#tag, &#ident, tm)
            },
        }
    }

    pub fn clear(&self, ident: TokenStream) -> TokenStream {
        match self.kind {
            Kind::Optional => quote!(#ident = ::core::option::Option::None),
            Kind::AlwaysEncode => panic!("oneof message field should not require clearing"),
            Kind::Repeated => quote!(#ident.clear()),
        }
    }
}
