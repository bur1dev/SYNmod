use hdi::prelude::*;

#[derive(Clone, PartialEq)]
#[hdk_entry_helper]
pub struct Product {
    pub name: String,
    pub price: f32,
    pub size: String,
    pub stocks_status: String,
    pub category: String,
    pub subcategory: Option<String>, // Add this
    pub image_url: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProductInput {
    pub product: Product,
    pub main_category: String,
    pub subcategory: Option<String>
}

pub fn validate_create_product(
    _action: EntryCreationAction,
    product: Product,
) -> ExternResult<ValidateCallbackResult> {
    if product.name.is_empty() {
        return Ok(ValidateCallbackResult::Invalid("Product name cannot be empty".into()));
    }
    if product.price < 0.0 {
        return Ok(ValidateCallbackResult::Invalid("Price cannot be negative".into()));
    }
    
    if let Some(url) = &product.image_url {
        if !url.contains("kroger.com") {
            return Ok(ValidateCallbackResult::Invalid("Invalid image URL".into()));
        }
    }
    
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_product(
    _action: Update,
    _product: Product,
    _original_action: EntryCreationAction,
    _original_product: Product,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(
        "Products cannot be updated".to_string(),
    ))
}

pub fn validate_delete_product(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_product: Product,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(
        "Products cannot be deleted".to_string(),
    ))
}

pub fn validate_create_link_products_by_category(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    _target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_link_products_by_category(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}