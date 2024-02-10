#[cfg(feature = "bot")]
mod components;

///
/// A macro that creates a [`Vec`] of [`serenity::builder::CreateActionRow`]s
/// from a list of buttons, select menus, or both.
///
/// # Example
///
/// ```rust
/// use keia_macros::discord_components;
///
/// let component = discord_component!(url!("https://example.com") => emoji!("") "label")
/// ```
#[cfg(feature = "bot")]
#[proc_macro]
pub fn component(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	components::discord_component_inner(item.into()).into()
}

///
/// A macro that creates a [`Vec`] of [`serenity::builder::CreateActionRow`]s
/// from a list of buttons, select menus, or both.
///
/// # Example
///
/// ```rust
/// use keia_macros::discord_components;
///
/// let component = discord_component!(url!("https://example.com") => emoji!("") "label")
/// ```
#[cfg(feature = "bot")]
#[proc_macro]
pub fn split_components(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	components::discord_components_inner(item.into()).into()
}
