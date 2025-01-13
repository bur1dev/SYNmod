use hdk::prelude::*;
use products_integrity::*;

#[hdk_extern]
pub fn add_to_favorites(product_hash: ActionHash) -> ExternResult<ActionHash> {
    let path = Path::from("private_favorites");
    create_link(
        path.path_entry_hash()?,
        product_hash,
        LinkTypes::Favorite,
        ()
    )
}

#[hdk_extern]
pub fn remove_from_favorites(product_hash: ActionHash) -> ExternResult<()> {
    let path = Path::from("private_favorites");
    let links = get_links(
        GetLinksInputBuilder::try_new(
            path.path_entry_hash()?, 
            LinkTypes::Favorite
        )?.build(),
    )?;
    
    let target_hash: AnyLinkableHash = product_hash.clone().into();
    for link in links {
        if link.target == target_hash {
            delete_link(link.create_link_hash)?;
        }
    }
    Ok(())
}

#[hdk_extern]
pub fn get_favorite_products(_: ()) -> ExternResult<Vec<Record>> {
    let path = Path::from("private_favorites");
    let links = get_links(
        GetLinksInputBuilder::try_new(
            path.path_entry_hash()?,
            LinkTypes::Favorite
        )?.build(),
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