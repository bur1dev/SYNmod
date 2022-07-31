import type {
  EntryHashB64,
  AgentPubKeyB64,
} from '@holochain-open-dev/core-types';
import { BinarySyncMessage } from 'automerge';

// Sent by the folk
export interface SendSyncRequestInput {
  sessionHash: EntryHashB64;
  to: AgentPubKeyB64;
  syncMessage: BinarySyncMessage | undefined;
  ephemeralSyncMessage: BinarySyncMessage | undefined;
}

// Received by the scribe
export interface RequestSyncInput {
  from: AgentPubKeyB64;
  to: AgentPubKeyB64;

  syncMessage: BinarySyncMessage | undefined;
  ephemeralSyncMessage: BinarySyncMessage | undefined;
}

export interface SyncResponseInput {
  participant: AgentPubKeyB64;
  sessionHash: EntryHashB64;
  syncMessage: BinarySyncMessage | undefined;
  ephemeralSyncMessage: BinarySyncMessage | undefined;
}