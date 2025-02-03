use hdk::prelude::*;
use products_integrity::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct CategorizedProducts {
    pub category: String,
    pub subcategory: Option<String>,
    pub product_type: Option<String>,
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
    pub product_type: Option<String>,
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
    debug!("Get Products Params: {:?}", params);
    
    let (base_path, link_type) = match (&params.subcategory, &params.product_type) {
        (Some(subcategory), Some(product_type)) => {
            let path = format!(
                "categories/{}/subcategories/{}/types/{}/products",
                params.category,
                subcategory,
                product_type
            );
            debug!("Using product type path: {}", path);
            (Path::try_from(path)?, LinkTypes::ProductTypeToProducts)
        },
        (Some(subcategory), None) => {
            let path = format!(
                "categories/{}/subcategories/{}/products",
                params.category,
                subcategory
            );
            debug!("Using subcategory path: {}", path);
            (Path::try_from(path)?, LinkTypes::ProductsByCategory)
        },
        _ => {
            let path = format!("categories/{}/products", params.category);
            debug!("Using category path: {}", path);
            (Path::try_from(path)?, LinkTypes::ProductsByCategory)
        }
    };

    let path_hash = base_path.path_entry_hash()?;
    debug!("Path hash: {:?}", path_hash);

    let links = get_links(
        GetLinksInputBuilder::try_new(
            path_hash,
            link_type
        )?.build()
    )?;

    debug!("Found {} links for path", links.len());

    let total = links.len();
    let start = params.offset;
    let end = (start + params.limit).min(total);
    let has_more = end < total;

    let mut products = Vec::new();
    for link in links.into_iter().skip(start).take(params.limit) {
        match get(link.target.clone().into_action_hash().unwrap(), GetOptions::default())? {
            Some(record) => products.push(record),
            None => debug!("Product not found for link: {:?}", link),
        }
    }

    debug!("Returning {} products", products.len());
    
    Ok(CategorizedProducts {
        category: params.category,
        subcategory: params.subcategory,
        product_type: params.product_type,
        products,
        total,
        has_more
    })
}

#[hdk_extern]
pub fn get_all_category_products(params: GetProductsParams) -> ExternResult<CategorizedProducts> {
    let products_path = Path::try_from(format!("categories/{}/products", params.category))?;

    let links = get_links(
        GetLinksInputBuilder::try_new(
            products_path.path_entry_hash()?, 
            LinkTypes::ProductsByCategory
        )?.build()
    )?;

    let total = links.len();
    let start = params.offset;
    let end = (start + params.limit).min(total);
    let has_more = end < total;

    let mut products = Vec::new();
    for link in links.into_iter().skip(start).take(params.limit) {
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
        product_type: None,
        products,
        total,
        has_more
    })
}