use ordinalizer::Ordinal;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parenthesized, parse::Parse, parse2, punctuated::Punctuated, Error, Expr, Lit, LitStr, Token
};

struct ButtonKind {
    custom_id: String,
    style: Option<Expr>,
}

struct UrlKind {
    url: Expr,
}

#[derive(Ordinal)]
enum ComponentKind {
    Row,
    URL(UrlKind),
    Button(ButtonKind),
}

impl Parse for ComponentKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kind: Ident = input.parse()?;

        let _: Token![!] = input.parse()?;

        let content;
        let _ = parenthesized!(content in input);

        match kind.to_string().as_str() {
            "row" => Ok(ComponentKind::Row),

            "url" => {
                let url: Expr = content.parse()?;
                Ok(ComponentKind::URL(UrlKind { url }))
            }

            "btn" => {
                let custom_id: LitStr = content.parse()?;

                let style = if content.parse::<Option<Token![,]>>()?.is_some() {
                    Some(content.parse()?)
                } else {
                    None
                };

                Ok(ComponentKind::Button(ButtonKind {
                    custom_id: custom_id.value(),
                    style,
                }))
            }

            _ => Err(Error::new(Span::call_site(), "expected url or btn")),
        }
    }
}

enum Emoji {
    Unicode(char),
    Custom(u64),
}

struct Label {
    value: String,
    emoji: Option<Emoji>,
}

impl Label {
    fn expand(&self) -> TokenStream {
        let value = &self.value;
        let emoji = self.expand_emoji();
        quote! {
            .label(#value)
            #emoji
        }
    }

    fn expand_emoji(&self) -> TokenStream {
        match self.emoji {
            Some(Emoji::Unicode(ch)) => quote! { .emoji(#ch) },
            Some(Emoji::Custom(id)) => quote! { .emoji(serenity::model::id::EmojiId::new(#id)) },
            None => quote! {},
        }
    }
}

impl Parse for Label {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut value = match input.parse::<Option<Ident>>()? {
            Some(ident) => ident.to_string(),
            None => input.parse::<LitStr>()?.value(),
        };

        let emoji = if value == "emoji" {
            let _ = input.parse::<Token![!]>()?;
            let content;
            let _ = parenthesized!(content in input);

            let emoji: Lit = content.parse()?;
            Some(match emoji {
                Lit::Int(int) => Emoji::Custom(int.base10_parse()?),
                Lit::Char(ch) => Emoji::Unicode(ch.value()),
                _ => return Err(syn::Error::new(emoji.span(), "expected emoji")),
            })
        } else {
            None
        };

        if emoji.is_some() {
            value = input.parse::<LitStr>()?.value();
        }

        Ok(Label { value, emoji })
    }
}

#[derive(Ordinal)]
enum Component {
    Button { kind: ButtonKind, label: Label },

    URL { kind: UrlKind, label: Label },

    Row,
}

impl Component {
    fn expand(&self) -> TokenStream {
        match self {
            Component::Button { kind, label } => {
                let custom_id = &kind.custom_id;
                let label = label.expand();
                let style = kind.style.as_ref().map(|style| quote! { .style(#style) });
                quote! { serenity::builder::CreateButton::new(#custom_id)
                #label
                #style }
            }

            Component::URL { kind, label } => {
                let url = &kind.url;
                let label = label.expand();
                quote! { serenity::builder::CreateButton::new_link(#url)#label }
            }

            Component::Row => Error::new(
                Span::call_site(),
                "cannot create new row outside of discord_components! macro",
            )
            .to_compile_error(),
        }
    }
}

impl Parse for Component {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kind: ComponentKind = input.parse()?;
        if kind.ordinal() == ComponentKind::Row.ordinal() {
            return Ok(Component::Row);
        }

        let _: Token![=>] = input.parse()?;

        Ok(match kind {
            ComponentKind::URL(kind) => {
                let label: Label = input.parse()?;
                Component::URL { kind, label }
            }

            ComponentKind::Button(kind) => {
                let label: Label = input.parse()?;
                Component::Button { kind, label }
            }

            _ => unreachable!()
        })
    }
}

pub fn discord_component_inner(item: TokenStream) -> TokenStream {
    let component = match parse2::<Component>(item) {
        Ok(functions) => functions,
        Err(err) => return err.to_compile_error(),
    };

    component.expand()
}

pub struct Components(Vec<Component>);

impl Components {
    pub fn expand(self) -> TokenStream {
        let mut rows: Vec<Vec<Component>> = Vec::new();
        let mut row: Vec<Component> = Vec::new();

        for component in self.0 {
            if component.ordinal() == Component::Row.ordinal() {
                if !row.is_empty() {
                    rows.push(row);
                    row = Vec::new();
                }

                continue;
            }

            if let Some(row_component) = row.first() {
                debug_assert!(component.ordinal() == row_component.ordinal());
            }

            row.push(component);
            if row.len() == 5 {
                rows.push(row);
                row = Vec::new();
            }
        }

        if !row.is_empty() {
            rows.push(row);
        }

        let rows = rows.into_iter().map(|row| {
            let row = row.into_iter().map(|component| component.expand());
            quote! { serenity::builder::CreateActionRow::Buttons(vec![#(#row),*]) }
        });

        quote! { vec![#(#rows),*] }
    }
}

impl Parse for Components {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let components = Punctuated::<Component, Token![,]>::parse_terminated(input)?;

        Ok(Components(components.into_iter().collect()))
    }
}

pub fn discord_components_inner(item: TokenStream) -> TokenStream {
    let components = match parse2::<Components>(item) {
        Ok(functions) => functions,
        Err(err) => return err.to_compile_error(),
    };

    components.expand()
}
