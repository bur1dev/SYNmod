use hdk::prelude::*;
use products_integrity::*;

#[hdk_extern]
pub fn create_product(input: CreateProductInput) -> ExternResult<Record> {
    let product_hash = create_entry(&EntryTypes::Product(input.product.clone()))?;

    // Create links for main category
    let category_path = Path::from(format!(
        "categories/{}/products", 
        input.main_category
    ));
    create_link(
        category_path.path_entry_hash()?,
        product_hash.clone(),
        LinkTypes::ProductsByCategory,
        ()
    )?;

    // Create links for subcategory if present
    if let Some(subcategory) = input.subcategory {
        let subcategory_path = Path::from(format!(
            "categories/{}/subcategories/{}/products",
            input.main_category,
            subcategory
        ));
        create_link(
            subcategory_path.path_entry_hash()?,
            product_hash.clone(),
            LinkTypes::ProductsByCategory,
            ()
        )?;
    }

    get(product_hash, GetOptions::default())?
        .ok_or(wasm_error!("Could not find the newly created Product"))
}

#[hdk_extern]
pub fn create_product_batch(products: Vec<CreateProductInput>) -> ExternResult<Vec<Record>> {
    let mut records = Vec::new();
    let mut errors = Vec::new();

    for input in products {
        match create_product(input) {
            Ok(record) => records.push(record),
            Err(e) => {
                warn!("Error creating product: {:?}", e);
                errors.push(e);
            }
        }
    }

    if !errors.is_empty() {
        warn!("Batch processing completed with {} errors", errors.len());
    }

    Ok(records)
}