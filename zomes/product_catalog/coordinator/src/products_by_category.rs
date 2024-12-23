use hdk::prelude::*;
use products_integrity::*;

#[hdk_extern]
pub fn get_products_by_category() -> ExternResult<Vec<Link>> {
    let path = Path::from("products_by_category");
    get_links(
        GetLinksInputBuilder::try_new(path.path_entry_hash()?, LinkTypes::ProductsByCategory)?
            .build(),
    )
}
