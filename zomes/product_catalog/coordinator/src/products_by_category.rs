use hdk::prelude::*;
use products_integrity::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct CategorizedProducts {
    pub category: String,
    pub subcategory: Option<String>,
    pub products: Vec<Record>,
    pub total: usize,
    pub has_more: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProductsParams {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize
}

fn default_limit() -> usize {
    20
}

#[hdk_extern]
pub fn get_products_by_category(params: GetProductsParams) -> ExternResult<CategorizedProducts> {
    let base_path = if let Some(ref subcategory) = params.subcategory {
        Path::try_from(format!("categories/{}/subcategories/{}/products", params.category, subcategory))?
    } else {
        Path::try_from(format!("categories/{}/products", params.category))?
    };

    let links = get_links(
        GetLinksInputBuilder::try_new(
            base_path.path_entry_hash()?, 
            LinkTypes::ProductsByCategory
        )?.build()
    )?;

    let total = links.len();
    let start = params.offset;
    let end = (start + params.limit).min(total);
    let has_more = end < total;

    let mut products = Vec::new();
    for link in links.into_iter().skip(start).take(params.limit) {
        if let Some(record) = get(link.target.into_action_hash().unwrap(), GetOptions::default())? {
            products.push(record);
        }
    }

    Ok(CategorizedProducts {
        category: params.category,
        subcategory: params.subcategory,
        products,
        total,
        has_more
    })
}

#[hdk_extern]
pub fn get_all_category_products(params: GetProductsParams) -> ExternResult<CategorizedProducts> {
    // Get base path for category products directly
    let products_path = Path::try_from(format!("categories/{}/products", params.category))?;

    let links = get_links(
        GetLinksInputBuilder::try_new(
            products_path.path_entry_hash()?, 
            LinkTypes::ProductsByCategory
        )?.build()
    )?;

    let total = links.len();
    let mut products = Vec::new();

    for link in links {
        if let Some(record) = get(
            link.target.into_action_hash().ok_or(wasm_error!("Invalid action hash"))?,
            GetOptions::default()
        )? {
            products.push(record);
        }
    }

    Ok(CategorizedProducts {
        category: params.category,
        subcategory: None,
        products,
        total,
        has_more: false
    })
}