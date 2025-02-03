use hdk::prelude::*;
use products_integrity::*;

#[hdk_extern]
pub fn create_product(input: CreateProductInput) -> ExternResult<Record> {
  debug!("Creating product: {:?}", input);
  let product_hash = create_entry(&EntryTypes::Product(input.product.clone()))?;

  // Create links for main category
  let category_path = Path::from(format!(
      "categories/{}/products", 
      input.main_category
  ));
  debug!("Creating main category link for: {}", input.main_category);
  create_link(
      category_path.path_entry_hash()?,
      product_hash.clone(),
      LinkTypes::ProductsByCategory,
      ()
  )?;

  // Create links for subcategory and product type if present
  if let Some(subcategory) = input.subcategory.as_ref() {
      let subcategory_path = Path::from(format!(
          "categories/{}/subcategories/{}/products",
          input.main_category,
          subcategory
      ));
      debug!("Creating subcategory link for: {}", subcategory);
      create_link(
          subcategory_path.path_entry_hash()?,
          product_hash.clone(),
          LinkTypes::ProductsByCategory,
          ()
      )?;

      // Create product type link if present
      if let Some(product_type) = input.product_type.as_ref() {
          let product_type_path = Path::from(format!(
              "categories/{}/subcategories/{}/types/{}/products",
              input.main_category,
              subcategory,
              product_type
          ));
          debug!("Creating product type link for: {}", product_type);
          create_link(
              product_type_path.path_entry_hash()?,
              product_hash.clone(),
              LinkTypes::ProductTypeToProducts,
              ()
          )?;
      }
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