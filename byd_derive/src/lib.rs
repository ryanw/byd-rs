use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(CastBytes)]
pub fn derive_as_bytes(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	impl_as_bytes(&ast)
}

fn impl_as_bytes(ast: &syn::DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let gen = quote! {
		unsafe impl bytemuck::Zeroable for #name {}
		unsafe impl bytemuck::Pod for #name {}
	};
	gen.into()
}
