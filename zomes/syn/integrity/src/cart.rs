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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CartCloneInfo {
    pub dna_hash: DnaHash,
    pub agent_key: AgentPubKey,
    pub cart_dna_hash: DnaHash,
    pub original_dna_hash: DnaHash,
    pub document_hash: AnyDhtHash,
    pub created_at: Timestamp,
}

#[hdk_entry_helper]
#[derive(Clone)]
pub struct CloneEntry {
    pub clone_info: CartCloneInfo,
}