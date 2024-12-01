use hdi::prelude::*;  // This brings in Serialize, Deserialize, etc.
use hdi::{hdk_entry_types, hdk_link_types};  // These bring in the attribute macros
// Add cart to the mod statements at the top
mod commit;
mod document;
mod workspace;
mod cart;  // New

// Add cart to the pub use statements
pub use commit::*;
pub use document::*;
pub use workspace::*;
pub use cart::*;  // New

// Add Cart to EntryTypes
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Document(Document),
    Workspace(Workspace),
    Commit(Commit),
    Cart(Cart),  // New
    CloneEntry(CloneEntry),
}

// Add cart-related link types
#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    TagToDocument,
    DocumentToAuthors,
    DocumentToWorkspaces,
    DocumentToCommits,
    WorkspaceToTip,
    WorkspaceToParticipant,
    CartToDocument,     // New: Links a cart to its parent document
    CartToParticipant, // New: Links a cart to authorized participants
    CartToSticky,  // Add this
    CartPath,
}