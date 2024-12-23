use hdk::prelude::*;
use products_integrity::*;

#[hdk_extern]
pub fn get_all_products() -> ExternResult<Vec<Record>> {
   let path = Path::from("all_products");
   let links = get_links(
       GetLinksInputBuilder::try_new(path.path_entry_hash()?, LinkTypes::AllProducts)?.build(),
   )?;
   
   let mut products = Vec::new();
   for link in links {
       if let Some(hash) = link.target.into_action_hash() {
           if let Ok(Some(record)) = get(hash, GetOptions::default()) {
               products.push(record);
           }
       }
   }
   Ok(products)
}
