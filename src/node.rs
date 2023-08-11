use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{discouraged::Speculative, Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, ExprTuple, Ident, Token,
};

use crate::keyword;

#[derive(Clone, Debug)]
pub struct Node {
    pub tag: Ident,
    pub fields_paren_token: Option<token::Paren>,
    pub fields: Punctuated<Field, Token![,]>,
    pub children_paren_token: Option<token::Paren>,
    pub children: Children,
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag = input.parse()?;

        let mut fields_paren_token = None;
        let mut fields = Punctuated::default();

        let mut children_paren_token = None;
        let mut children = Children::default();

        let mut parsed_children = false;

        if input.peek(token::Paren) {
            // Try parse attrs
            let content;
            let paren_token = parenthesized!(content in input);
            let fork = content.fork();
            match fork.parse_terminated(Field::parse, Token![,]) {
                Ok(new_attrs) => {
                    fields_paren_token = Some(paren_token);
                    fields = new_attrs;
                    content.advance_to(&fork);
                }
                Err(attrs_err) => {
                    // Attrs failed, lets try children
                    parsed_children = true;

                    children = input.parse().map_err(|children_err| {
                        let mut err =
                            syn::Error::new(content.span(), "expected attributes or children");
                        err.combine(attrs_err);
                        err.combine(children_err);
                        err
                    })?;
                }
            }
        }

        if input.peek(token::Paren) && !parsed_children {
            // Try parse children
            let content;
            children_paren_token = Some(parenthesized!(content in input));
            children = content.parse()?;
        }

        Ok(Node {
            tag,
            fields_paren_token,
            fields,
            children_paren_token,
            children,
        })
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            tag,
            fields,
            children,
            ..
        } = self;

        tokens.extend(quote! {
            leptos::html::#tag()
        });

        for field in fields {
            field.to_tokens(tokens);
        }
        children.to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
pub enum Field {
    Attr(Attr),
    Class(Class),
    // Event(Event),
    // Id(Id),
    // Ref(Ref),
    Style(Style),
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::class) {
            Ok(Field::Class(input.parse()?))
        } else if input.peek(keyword::style) {
            Ok(Field::Style(input.parse()?))
        } else {
            Ok(Field::Attr(input.parse()?))
        }
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Field::Attr(attr) => attr.to_tokens(tokens),
            Field::Class(class) => class.to_tokens(tokens),
            Field::Style(style) => style.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Attr {
    pub name: Ident,
    pub equals_token: Token![=],
    pub value: Expr,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Attr {
            name: input.parse()?,
            equals_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for Attr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { name, value, .. } = self;

        let name = name.to_string();
        tokens.extend(quote! {
            .attr(#name, #value)
        });

        // let expanded = if name == "class" {
        //     quote! {
        //         .classes(#value)
        //     }
        // } else if name == "id" {
        //     quote! {
        //         .id(#value)
        //     }
        // } else if name == "_ref" {
        //     quote! {
        //         .node_ref(#value)
        //     }
        // } else if name == "style" {
        //     quote! {
        //         .style(#value)
        //     }
        // } else {
        //     let name = name.to_string();
        //     quote! {
        //         .attr(#name, #value)
        //     }
        // };

        // tokens.extend(expanded);
    }
}

#[derive(Clone, Debug)]
pub struct Class {
    pub name: keyword::class,
    pub equals_token: Token![=],
    pub value: ClassValue,
}

impl Parse for Class {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Class {
            name: input.parse()?,
            equals_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for Class {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expanded = match &self.value {
            ClassValue::Static(value) => {
                quote! {
                    .classes(#value)
                }
            }
            ClassValue::Dynamic(name, class) => {
                quote! {
                    .class(#name, #class)
                }
            }
        };

        tokens.extend(expanded);
    }
}

#[derive(Clone, Debug)]
pub enum ClassValue {
    Static(Expr),
    Dynamic(Expr, Expr),
}

impl Parse for ClassValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        match fork.parse::<ExprTuple>() {
            Ok(tuple) => {
                input.advance_to(&fork);

                let tuple_len = tuple.elems.len();
                let mut tuple_elems = tuple.elems.into_iter();
                let name = tuple_elems
                    .next()
                    .ok_or_else(|| fork.error("expected a tuple with 2 items"))?;
                let class = tuple_elems
                    .next()
                    .ok_or_else(|| fork.error("expected a tuple with 2 items"))?;

                if tuple_elems.next().is_some() {
                    return Err(fork.error(format!("tuple has {} items, expected 2", tuple_len)));
                }

                Ok(ClassValue::Dynamic(name, class))
            }
            Err(_) => Ok(ClassValue::Static(input.parse()?)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Style {
    pub name: keyword::style,
    pub equals_token: Token![=],
    pub value: Expr,
}

impl Parse for Style {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Style {
            name: input.parse()?,
            equals_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { value, .. } = self;

        tokens.extend(quote! {
            .style(#value)
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Children(Punctuated<Child, Token![,]>);

impl Parse for Children {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Children(Punctuated::default()));
        }

        Ok(Children(input.parse_terminated(Child::parse, Token![,])?))
    }
}

impl ToTokens for Children {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for child in &self.0 {
            child.to_tokens(tokens);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Child {
    pub expr: Expr,
}

impl Parse for Child {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Child {
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Child {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { expr } = self;
        tokens.extend(quote! {
           .child(#expr)
        });
    }
}
