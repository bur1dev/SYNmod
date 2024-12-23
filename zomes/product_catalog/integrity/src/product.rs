use hdi::prelude::*;

#[derive(Clone, PartialEq)]
#[hdk_entry_helper]
pub struct Product {
    pub name: String,
    pub price: f32,
    pub size: String,
    pub stocks_status: String,
    pub category: String,
    pub image_url: Option<String>
}

pub fn validate_create_product(
    _action: EntryCreationAction,
    product: Product,
) -> ExternResult<ValidateCallbackResult> {
    // Validate required fields
    if product.name.is_empty() {
        return Ok(ValidateCallbackResult::Invalid("Product name cannot be empty".into()));
    }
    if product.price < 0.0 {
        return Ok(ValidateCallbackResult::Invalid("Price cannot be negative".into()));
    }
    
    // Validate image URL if present
    if let Some(url) = &product.image_url {
        if !url.starts_with("https://www.kroger.com/") {
            return Ok(ValidateCallbackResult::Invalid("Image URL must be from kroger.com domain".into()));
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

pub fn validate_create_link_all_products(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let action_hash =
        target_address
            .into_action_hash()
            .ok_or(wasm_error!(WasmErrorInner::Guest(
                "No action hash associated with link".to_string()
            )))?;
    let record = must_get_valid_record(action_hash)?;
    let _product: crate::Product = record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(wasm_error!(WasmErrorInner::Guest(
            "Linked action must reference an entry".to_string()
        )))?;
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_link_all_products(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_create_link_products_by_category(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let action_hash =
        target_address
            .into_action_hash()
            .ok_or(wasm_error!(WasmErrorInner::Guest(
                "No action hash associated with link".to_string()
            )))?;
    let record = must_get_valid_record(action_hash)?;
    let _product: crate::Product = record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(wasm_error!(WasmErrorInner::Guest(
            "Linked action must reference an entry".to_string()
        )))?;
    // TODO: add the appropriate validation rules
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
