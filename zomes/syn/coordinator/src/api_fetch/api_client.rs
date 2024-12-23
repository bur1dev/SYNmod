use hc_zome_syn_integrity::{Document, EntryTypes};
use hdk::prelude::*;
use serde::{Deserialize, Serialize};

// This struct matches the full API response
#[derive(Serialize, Deserialize, Debug)]
pub struct Price {
    pub regular: f64,
    pub promo: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub item_id: String,
    pub price: Price,
    pub size: String,
    #[serde(default)]
    pub by: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub product_id: String,
    pub brand: String,
    pub description: String,
    pub categories: Vec<String>,
    #[serde(default)]
    pub items: Vec<Item>,
}

// This is our simplified struct for sticky display
#[derive(Serialize, Deserialize, Debug)]
pub struct RetailProduct {
    pub name: String,        // e.g., "fairlife Whole Ultra-Filtered Milk"
    pub price: f64,         // e.g., 5.99
    pub size: String,       // e.g., "52 fl oz"
    pub stock_status: String, // e.g., "Many in stock"
    pub category: String,    // e.g., "Dairy"
}

// Function to create a document from our RetailProduct
pub fn create_product_document(retail_product: RetailProduct) -> ExternResult<Record> {
    let product_bytes = serde_json::to_vec(&retail_product)
        .map_err(|err| wasm_error!(WasmErrorInner::Guest(err.to_string())))?;
    
    let document = Document {
        initial_state: SerializedBytes::from(UnsafeBytes::from(product_bytes)),
        meta: None,
    };

    let document_hash = create_entry(EntryTypes::Document(document.clone()))?;
    let record = get(document_hash, GetOptions::default())?
        .ok_or(wasm_error!(WasmErrorInner::Guest("Document not found".into())))?;

    Ok(record)
}