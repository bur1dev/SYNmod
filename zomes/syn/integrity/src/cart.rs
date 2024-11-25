use hdi::prelude::*;

#[hdk_entry_helper]
#[derive(Clone)]
pub struct Cart {
    pub original_dna_hash: DnaHash,    
    pub cart_dna_hash: DnaHash,        
    pub document_hash: AnyDhtHash,     
    pub owner: AgentPubKey,           
    pub status: CartStatus,           
    pub created_at: Timestamp,
    pub meta: Option<SerializedBytes>, 
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CartStatus {
    Active,
    CheckedOut,
    Processed,
}