use hdk::prelude::*;
use hc_zome_syn_integrity::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCallZomeRequest {
    pub role_name: String,
    pub zome_name: String,
    pub fn_name: String,
    pub payload: DeleteCartInput,
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteCartInput {
    pub cart_id: String,
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

    let cart_name = format!("Cart_{}", created_at.to_string().split('.').next().unwrap_or(""));
    warn!("Registering cell with role_name: {}", cart_name);
    let clone_cell = CreateCloneCellInput {
        cell_id: CellId::new(dna.hash.clone(), agent.agent_initial_pubkey.clone()),
        membrane_proof: Some(membrane_proof),
        name: Some(cart_name.clone()),

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
    warn!("Clone cell result: {:?}", cloned);

    // Store clone info
    let clone_info = CartCloneInfo {
        dna_hash: dna.hash.clone(),
        agent_key: cloned.cell_id.agent_pubkey().clone(),
        cart_dna_hash: cloned.cell_id.dna_hash().clone(),
        original_dna_hash: dna.hash.clone(),
        document_hash: input.document_hash.clone(),
        created_at,
        role_name: cart_name.clone(),
        holochain_clone_id: cloned.clone_id.to_string(),
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
    
    let path = Path::from("cart_clones");
    let links = get_links(GetLinksInputBuilder::try_new(
        path.path_entry_hash()?,
        LinkTypes::CartToDocument,
    )?.build())?;
    
    let mut clone_dna = None;
    for link in links {
        if let Some(action_hash) = link.target.into_action_hash() {
            if let Some(record) = get(action_hash, GetOptions::default())? {
                if let Some(clone_entry) = record.entry().to_app_option::<CloneEntry>()
                    .map_err(|e| wasm_error!(WasmErrorInner::Guest(format!("Serialization error: {}", e))))? {
                    clone_dna = Some(clone_entry.clone_info.cart_dna_hash);
                    break;
                }
            }
        }
    }

    let cart = Cart {
        original_dna_hash: dna.hash.clone(),
        cart_dna_hash: clone_dna.unwrap_or(dna.hash),
        document_hash: entry_input.input.document_hash.clone(),
        owner: agent.agent_initial_pubkey.clone(),
        status: CartStatus::Active,
        created_at: entry_input.created_at,
        cart_name: format!("Cart_{}", entry_input.created_at.to_string().split('.').next().unwrap_or("")),
        meta: None,
    };

    let action_hash = create_entry(EntryTypes::Cart(cart.clone()))?;
    
    let agent_path = Path::from(format!("agent_carts_{}", agent.agent_initial_pubkey))
        .typed(LinkTypes::CartPath)?;
    
    create_link(
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
                if let Some(cart) = record.entry().to_app_option::<Cart>()
    .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))? {
                    if cart.status == CartStatus::Active {
                        carts.push(record);
                    }
                }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CloneCell {
    pub dna_hash: DnaHash,
    pub agent_key: AgentPubKey,
}

#[hdk_extern]
pub fn get_cell_for_cart(cart_id: String) -> ExternResult<CellId> {
    let agent_path = Path::from(format!("agent_carts_{}", agent_info()?.agent_initial_pubkey))
        .typed(LinkTypes::CartPath)?;
    
    let links = get_links(GetLinksInputBuilder::try_new(
        agent_path.path_entry_hash()?,
        LinkTypes::CartToDocument,
    )?.build())?;
    
    for link in links {
        if let Some(cart_hash) = link.target.into_action_hash() {
            if let Some(record) = get(cart_hash, GetOptions::default())? {
                if let Some(cart) = record.entry().to_app_option::<Cart>()
                    .map_err(|e| wasm_error!(WasmErrorInner::Guest(format!("Serialization error: {}", e))))? 
                {
                    if cart_id.contains(&cart.cart_name) {
                        return Ok(CellId::new(cart.cart_dna_hash, cart.owner));
                    }
                }
            }
        }
    }
    
    Err(wasm_error!("Cart not found"))
}

#[hdk_extern]
pub fn delete_cart(input: DeleteCartInput) -> ExternResult<()> {
  let path = Path::from("cart_clones");
  let clone_links = get_links(GetLinksInputBuilder::try_new(
      path.path_entry_hash()?,
      LinkTypes::CartToDocument,
  )?.build())?;

  let parts: Vec<&str> = input.cart_id.split('_').collect();
  let cart_dna_hash = parts[1..parts.len()-1].join("_");
  let cart_timestamp = parts[parts.len()-1];
  warn!("Cart ID from input: {}", input.cart_id);
  warn!("Extracted DNA hash: {}", cart_dna_hash);
  warn!("Cart timestamp: {}", cart_timestamp);

  let mut target_clone_entry = None;
  let mut target_clone_link = None;

  for link in clone_links.iter() {
      if let Some(action_hash) = link.target.clone().into_action_hash() {
          if let Some(record) = get(action_hash, GetOptions::default())? {
              let clone_entry = record
                  .entry()
                  .to_app_option::<CloneEntry>()
                  .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?
                  .ok_or(wasm_error!("Expected CloneEntry"))?;

              let clone_timestamp = clone_entry.clone_info.created_at.as_micros().to_string();
              warn!("Comparing - Input hash: {}, Stored hash: {}", cart_dna_hash, clone_entry.clone_info.cart_dna_hash.to_string());
              warn!("Comparing - Input timestamp: {}, Stored timestamp: {}", cart_timestamp, clone_timestamp);

              if clone_entry.clone_info.cart_dna_hash.to_string() == cart_dna_hash && clone_timestamp == cart_timestamp {
                  target_clone_entry = Some(clone_entry);
                  target_clone_link = Some(link.clone());
                  break;
              }
          }
      }
  }

  if let Some(clone_entry) = target_clone_entry {
      let agent_path = Path::from(format!("agent_carts_{}", agent_info()?.agent_initial_pubkey))
          .typed(LinkTypes::CartPath)?;
      
      let cart_links = get_links(GetLinksInputBuilder::try_new(
          agent_path.path_entry_hash()?,
          LinkTypes::CartToDocument,
      )?.build())?;

      for cart_link in cart_links {
          if let Some(cart_hash) = cart_link.target.clone().into_action_hash() {
              if let Some(cart_record) = get(cart_hash, GetOptions::default())? {
                  if let Some(mut cart) = cart_record.entry().to_app_option::<Cart>()
                      .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))? {
                      let cart_created_timestamp = cart.created_at.as_micros().to_string();
                      if cart.cart_dna_hash.to_string() == cart_dna_hash && cart_created_timestamp == cart_timestamp {
                          cart.status = CartStatus::Processed;
                          let new_action_hash = create_entry(EntryTypes::Cart(cart))?;
                          
                          create_link(
                              agent_path.path_entry_hash()?,
                              new_action_hash,
                              LinkTypes::CartToDocument,
                              (),
                          )?;
                          delete_link(cart_link.create_link_hash)?;

                          let clone_id = CloneId::try_from(clone_entry.clone_info.holochain_clone_id.clone())
                              .map_err(|_| wasm_error!("Invalid clone ID format"))?;

                          HDK.with(|hdk| {
                              let result = hdk.borrow().disable_clone_cell(DisableCloneCellInput {
                                  clone_cell_id: CloneCellId::CloneId(clone_id.clone())
                              });

                              match result {
                                  Ok(_) => {
                                      hdk.borrow().delete_clone_cell(DeleteCloneCellInput {
                                          clone_cell_id: CloneCellId::CloneId(clone_id)
                                      })?;
                                      if let Some(target_link) = target_clone_link {
                                          delete_link(target_link.create_link_hash)?;
                                      }
                                      Ok(())
                                  },
                                  Err(e) => Err(e)
                              }
                          })?;

                          return Ok(());
                      }
                  }
              }
          }
      }
  }
  
  Err(wasm_error!("Cart not found"))
}

fn random_network_seed() -> ExternResult<String> {
    let random_bytes = random_bytes(32)?;
    Ok(base64::encode(&random_bytes))
}