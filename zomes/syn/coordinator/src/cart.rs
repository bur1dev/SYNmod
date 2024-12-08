use hdk::prelude::*;
use hc_zome_syn_integrity::*;
use crate::utils::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct CloneCartInput {
    pub document_hash: AnyDhtHash,
    pub cart_name: String,
    pub created_at: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloneInfo {
    pub cell_id: CellId,
    pub original_dna_hash: DnaHash,
    pub cart_dna_hash: DnaHash,
    pub created_at: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCartEntryInput {
    pub input: CloneCartInput,
    pub created_at: Timestamp,
}

#[hdk_extern]
pub fn clone_cart_dna(input: CloneCartInput) -> ExternResult<CloneInfo> {
    let dna = dna_info()?;
    let agent = agent_info()?;
    let network_seed = random_network_seed()?;
    let created_at = sys_time()?;

    warn!("Cloning DNA with seed: {:?}", network_seed);
    warn!("DNA info: {:?}", dna);
    
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

    // Store clone info
    let clone_info = CartCloneInfo {
        dna_hash: dna.hash.clone(),
        agent_key: cloned.cell_id.agent_pubkey().clone(),
        cart_dna_hash: cloned.cell_id.dna_hash().clone(),
        original_dna_hash: dna.hash.clone(),
        document_hash: input.document_hash.clone(),
        created_at,
    };

    let clone_entry = CloneEntry { clone_info: clone_info.clone() };
    let action_hash = create_entry(EntryTypes::CloneEntry(clone_entry))?;
    
    // Create link from document to clone info
    let path = Path::from("cart_clones");
    create_link(
        path.path_entry_hash()?,
        action_hash,
        LinkTypes::CartToDocument,
        ()
    )?;

    Ok(CloneInfo {
        cell_id: cloned.cell_id.clone(),
        original_dna_hash: dna.hash,
        cart_dna_hash: cloned.cell_id.dna_hash().clone(),
        created_at,
    })
}

#[hdk_extern]
pub fn get_cart_clones(_: ()) -> ExternResult<Vec<CartCloneInfo>> {
    let path = Path::from("cart_clones");
    let links = get_links(GetLinksInputBuilder::try_new(
        path.path_entry_hash()?,
        LinkTypes::CartToDocument,
    )?.build())?;
    
    let mut clones = Vec::new();
    for link in links {
        if let Some(action_hash) = link.target.into_action_hash() {
            if let Some(record) = get(action_hash, GetOptions::default())? {
                let entry = record
                    .entry()
                    .to_app_option::<CloneEntry>()
                    .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;
                if let Some(entry) = entry {
                    clones.push(entry.clone_info);
                }
            }
        }
    }
    
    Ok(clones)
}

#[hdk_extern]
pub fn create_cart_entry(entry_input: CreateCartEntryInput) -> ExternResult<Record> {
    let dna = dna_info()?;
    let agent = agent_info()?;
    
    let cart = Cart {
        original_dna_hash: dna.hash.clone(),
        cart_dna_hash: dna.hash,
        document_hash: entry_input.input.document_hash.clone(),
        owner: agent.agent_initial_pubkey.clone(),
        status: CartStatus::Active,
        created_at: entry_input.created_at,
        meta: None,
    };

    let action_hash = create_relaxed(
        EntryTypes::Cart(cart.clone()),
        Entry::try_from(cart.clone())?
    )?;
    
    let agent_path = Path::from(format!("agent_carts_{}", agent.agent_initial_pubkey))
        .typed(LinkTypes::CartPath)?;
    
    create_link_relaxed(
        agent_path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::CartToDocument,
        (),
    )?;

    get(action_hash, GetOptions::default())?
        .ok_or(wasm_error!("Could not get the record created just now"))
}
    


#[hdk_extern]
pub fn get_all_carts() -> ExternResult<Vec<Record>> {
    let agent = agent_info()?;
    let base_path = Path::from(format!("agent_carts_{}", agent.agent_initial_pubkey))
        .typed(LinkTypes::CartPath)?
        .path_entry_hash()?;

    let links = get_links(GetLinksInputBuilder::try_new(
        base_path,
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

#[hdk_extern]
pub fn get_cart_contents() -> ExternResult<Vec<Record>> {
    let path = Path::from("cart_clones");
    let links = get_links(GetLinksInputBuilder::try_new(
        path.path_entry_hash()?,
        LinkTypes::CartToDocument,
    )?.build())?;
    
    let mut carts = Vec::new();
    for link in links {
        if let Some(action_hash) = link.target.into_action_hash() {
            if let Some(record) = get(action_hash, GetOptions::default())? {
                let entry = record
                    .entry()
                    .to_app_option::<CloneEntry>()
                    .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?
                    .ok_or(wasm_error!("Expected CloneEntry"))?;
                
                let cart_path = Path::from(format!("agent_carts_{}", entry.clone_info.agent_key))
                    .typed(LinkTypes::CartPath)?
                    .path_entry_hash()?;
                
                let cart_links = get_links(GetLinksInputBuilder::try_new(
                    cart_path,
                    LinkTypes::CartToDocument,
                )?.build())?;

                for cart_link in cart_links {
                    if let Some(cart_hash) = cart_link.target.into_action_hash() {
                        if let Some(cart_record) = get(cart_hash, GetOptions::default())? {
                            carts.push(cart_record);
                        }
                    }
                }
            }
        }
    }
    Ok(carts)
}

fn random_network_seed() -> ExternResult<String> {
    let random_bytes = random_bytes(32)?;
    Ok(base64::encode(&random_bytes))
}