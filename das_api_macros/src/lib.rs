extern crate proc_macro;

use convert_case::{Casing, Case};
use inflector::cases::snakecase::is_snake_case;
use proc_macro::{TokenStream};
use proc_macro2::Span;
use syn::{parse_macro_input, Ident, DeriveInput, Data, __private::quote::quote, spanned::Spanned, Error};


#[proc_macro_derive(DasApiFilter)]
pub fn derive_filter(input: TokenStream) -> TokenStream {
    
    // Parse input, storing name and generics
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_generics = &input.generics;


    // Get field names
    let fields = match &input.data {
        Data::Struct(struct_data) => &struct_data.fields,
        _ => panic!("packed_bools can only be used on a struct"),
    };

    // Iterate through all fields to get relevant token streams
    let (count_streams, cond_streams): (Vec<TokenStream>, Vec<TokenStream>) = fields
        .into_iter()
        .map(|f| {
            
            // Get field name and type
            let ref name = f.ident;
            let ref ty = f.ty;

            // Check that field name is in snake case
            if !is_snake_case(
                &name
                    .ok_or(
                        Error::new(name.span(), "something very bad happened and you have an invalid field name")
                        .into_compile_error())
                    .unwrap(/* a compile error was generated */)
                    .to_string()) {
                
                // A (compiler) valid field name is present, but it is not snake case
                Error::new(name.span(), "DasApiFilter requires fields to be in snake case")
                    .to_compile_error();
            }

            // Get field's inner attribute ident
            let ident: Option<Ident> = f
                .attrs
                .into_iter()
                .find(|attr| {

                    // This is constructed here so that it can have the attr's span
                    let valid_fields: Vec<Ident> = vec![
                        Ident::new("asset", attr.span()),
                        Ident::new("asset_creators", attr.span()),
                        Ident::new("asset_authority", attr.span()),
                        Ident::new("asset_grouping", attr.span()),
                    ];

                    // Get the (first) attribute contained in valid fields
                    valid_fields.contains(&attr
                        .path
                        .segments
                        .first()
                        .expect(&format!("expecting a valid (supported) attribute: {:?}", valid_fields))
                        .ident)

                    }).map(|attr| attr.path.segments.first().unwrap(/* already checked above */).ident);

            // Get snake and camel
            let snake_and_camel: Option<(Ident, Ident)> = ident
                .map(|x| {
                    // Return original (already-validated) snake case, as well as camel case ident
                    (x, Ident::new(x.to_string().to_case(Case::Camel).as_str(), x.span()))
                });

            // If this field was marked, geenrate token streams
            if let Some((snake, camel)) = snake_and_camel {

            }

            quote! {
                .add_option(a)
            }
            
        }).collect();


}