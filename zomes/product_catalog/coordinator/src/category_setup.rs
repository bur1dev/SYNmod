use hdk::prelude::*;
use products_integrity::LinkTypes;

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorySetup {
    pub main_category: String,
    pub subcategories: Vec<String>
}

#[hdk_extern]
pub fn create_category_structure(categories: Vec<CategorySetup>) -> ExternResult<()> {
    for category in categories {
        let category_path = Path::try_from(format!("categories/{}", category.main_category))?;
        let _products_path = Path::try_from(format!("categories/{}/products", category.main_category))?;

        for subcategory in category.subcategories {
            let subcategory_path = Path::try_from(format!(
                "categories/{}/subcategories/{}", 
                category.main_category, 
                subcategory
            ))?;
            let _sub_products_path = Path::try_from(format!(
                "categories/{}/subcategories/{}/products", 
                category.main_category,
                subcategory
            ))?;

            create_link(
                category_path.path_entry_hash()?,
                subcategory_path.path_entry_hash()?,
                LinkTypes::CategoryToSubcategory,
                ()
            )?;
        }
    }
    Ok(())
}