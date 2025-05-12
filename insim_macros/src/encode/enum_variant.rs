#[derive(Debug, darling::FromVariant)]
#[darling(attributes(insim))]
pub(super) struct Variant {
    pub ident: syn::Ident,
    pub discriminant: Option<syn::Expr>,
    pub skip: Option<bool>,
}

impl Variant {
    pub(super) fn skip(&self) -> bool {
        self.skip.unwrap_or(false)
    }
}
