use hdk::prelude::*;
use hc_zome_syn_integrity::*;
use crate::utils::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct CloneCartInput {
    pub document_hash: AnyDhtHash,
    pub cart_name: String,
}

#[hdk_extern]
pub fn create_cart(input: CloneCartInput) -> ExternResult<Record> {
    let dna = dna_info()?;
    let agent = agent_info()?;
    let network_seed = random_network_seed()?;
    
    let membrane_proof: MembraneProof = SerializedBytes::from(
        UnsafeBytes::from(agent.agent_initial_pubkey.get_raw_39().to_vec())
    ).into();

    let clone_cell = CreateCloneCellInput {
        cell_id: CellId::new(dna.hash.clone(), agent.agent_initial_pubkey.clone()),
        membrane_proof: Some(membrane_proof),
        name: Some(input.cart_name.clone()),
        modifiers: DnaModifiersOpt {
            network_seed: Some(network_seed.clone()),
            properties: None,
            origin_time: None,
            quantum_time: None,
        },
    };

    let cloned = HDK.with(|hdk| {
        hdk.borrow().create_clone_cell(clone_cell)
    })?;

    let cart = Cart {
        original_dna_hash: dna.hash,
        cart_dna_hash: cloned.cell_id.dna_hash().clone(),
        document_hash: input.document_hash.clone(),
        owner: agent.agent_initial_pubkey.clone(),
        status: CartStatus::Active,
        created_at: sys_time()?,
        meta: None,
    };

    let action_hash = create_relaxed(
        EntryTypes::Cart(cart.clone()),
        Entry::try_from(cart.clone())?
    )?;

    create_link_relaxed(
        action_hash.clone(),
        input.document_hash,
        LinkTypes::CartToDocument,
        (),
    )?;

    let record = get(action_hash, GetOptions::default())?;
    record.ok_or(wasm_error!("Could not get the record created just now"))
}

// Don't forget the helper functions
fn random_network_seed() -> ExternResult<String> {
    let random_bytes = random_bytes(32)?;
    Ok(base64::encode(&random_bytes))
}

#[hdk_extern]
pub fn get_all_carts() -> ExternResult<Vec<Record>> {
    let path = Path::from("all_carts");
    let links = get_links(GetLinksInputBuilder::try_new(
        path.path_entry_hash()?,
        LinkTypes::CartToDocument,
    )?.build())?;
    
    let mut carts = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                carts.push(record);
            }
        }
    }
    
    Ok(carts)
}