use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{discouraged::Speculative, Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Token,
};

#[derive(Clone, Debug)]
pub struct Node {
    pub tag: Ident,
    pub attrs_paren_token: Option<token::Paren>,
    pub attrs: Punctuated<Attr, Token![,]>,
    pub children_paren_token: Option<token::Paren>,
    pub children: Children,
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag = input.parse()?;

        let mut attrs_paren_token = None;
        let mut attrs = Punctuated::default();

        let mut children_paren_token = None;
        let mut children = Children::default();

        let mut parsed_children = false;

        if input.peek(token::Paren) {
            // Try parse attrs
            let content;
            let paren_token = parenthesized!(content in input);
            let fork = content.fork();
            match fork.parse_terminated(Attr::parse, Token![,]) {
                Ok(new_attrs) => {
                    attrs_paren_token = Some(paren_token);
                    attrs = new_attrs;
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
            attrs_paren_token,
            attrs,
            children_paren_token,
            children,
        })
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            tag,
            attrs,
            children,
            ..
        } = self;

        tokens.extend(quote! {
            leptos::html::#tag()
        });

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        children.to_tokens(tokens);
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

        let expanded = if name == "class" {
            quote! {
                .classes(#value)
            }
        } else if name == "id" {
            quote! {
                .id(#value)
            }
        } else if name == "_ref" {
            quote! {
                .node_ref(#value)
            }
        } else if name == "style" {
            quote! {
                .style(#value)
            }
        } else {
            let name = name.to_string();
            quote! {
                .attr(#name, #value)
            }
        };

        tokens.extend(expanded);
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
