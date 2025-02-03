use hdk::prelude::*;
use products_integrity::LinkTypes;

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorySetup {
    pub main_category: String,
    pub subcategories: Vec<SubcategorySetup>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubcategorySetup {
    pub name: String,
    pub product_types: Vec<String>
}

#[hdk_extern]
pub fn create_category_structure(categories: Vec<CategorySetup>) -> ExternResult<()> {
   for category in categories {
       let category_path = Path::try_from(format!("categories/{}", category.main_category))?;
       let category_path_clone = category_path.clone();
       category_path.typed(LinkTypes::ProductsByCategory)?.ensure()?;
       let _products_path = Path::try_from(format!("categories/{}/products", category.main_category))?;
       _products_path.typed(LinkTypes::ProductsByCategory)?.ensure()?;

       for subcategory in category.subcategories {
           let subcategory_path = Path::try_from(format!(
               "categories/{}/subcategories/{}", 
               category.main_category, 
               subcategory.name
           ))?;
           let subcategory_path_clone = subcategory_path.clone();
           subcategory_path.typed(LinkTypes::CategoryToSubcategory)?.ensure()?;
           
           let _sub_products_path = Path::try_from(format!(
               "categories/{}/subcategories/{}/products", 
               category.main_category,
               subcategory.name
           ))?;
           _sub_products_path.typed(LinkTypes::ProductsByCategory)?.ensure()?;

           create_link(
               category_path_clone.path_entry_hash()?,
               subcategory_path_clone.path_entry_hash()?,
               LinkTypes::CategoryToSubcategory,
               ()
           )?;

           for product_type in subcategory.product_types {
               let product_type_path = Path::try_from(format!(
                   "categories/{}/subcategories/{}/types/{}/products",
                   category.main_category,
                   subcategory.name,
                   product_type
               ))?;
               let product_type_path_clone = product_type_path.clone();
               product_type_path.typed(LinkTypes::ProductTypeToProducts)?.ensure()?;
               
               create_link(
                   subcategory_path_clone.path_entry_hash()?,
                   product_type_path_clone.path_entry_hash()?,
                   LinkTypes::ProductTypeToProducts,
                   ()
               )?;
           }
       }
   }
   Ok(())
}