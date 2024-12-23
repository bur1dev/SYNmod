use hdk::prelude::*;
use products_integrity::*;

#[hdk_extern]
pub fn create_product(product: Product) -> ExternResult<Record> {
    let product_hash = create_entry(&EntryTypes::Product(product.clone()))?;
    let record = get(product_hash.clone(), GetOptions::default())?.ok_or(wasm_error!(
        WasmErrorInner::Guest("Could not find the newly created Product".to_string())
    ))?;
    let path = Path::from("all_products");
    create_link(
        path.path_entry_hash()?,
        product_hash.clone(),
        LinkTypes::AllProducts,
        (),
    )?;
    let path = Path::from("products_by_category");
    create_link(
        path.path_entry_hash()?,
        product_hash.clone(),
        LinkTypes::ProductsByCategory,
        (),
    )?;
    Ok(record)
}

#[hdk_extern]
pub fn get_product(product_hash: ActionHash) -> ExternResult<Option<Record>> {
    let Some(details) = get_details(product_hash, GetOptions::default())? else {
        return Ok(None);
    };
    match details {
        Details::Record(details) => Ok(Some(details.record)),
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Malformed get details response".to_string()
        ))),
    }
}
