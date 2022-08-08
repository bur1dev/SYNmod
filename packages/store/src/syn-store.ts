import { derived, Writable, writable } from 'svelte/store';
import { Commit, SynClient, Workspace } from '@holochain-syn/client';
import { decode, encode } from '@msgpack/msgpack';
import { Create, EntryHash } from '@holochain/client';
import merge from 'lodash-es/merge';

import { defaultConfig, RecursivePartial, SynConfig } from './config';
import type { SynGrammar } from './grammar';
import { EntryHashMap } from '@holochain-open-dev/utils';
import { WorkspaceStore } from './workspace-store';
import { change, Doc, init, save } from 'automerge';

export class SynStore {
  /** Public accessors */
  knownWorkspaces: Writable<EntryHashMap<Workspace>> = writable(
    new EntryHashMap()
  );
  knownCommits: Writable<EntryHashMap<Commit>> = writable(new EntryHashMap());

  constructor(
    protected client: SynClient,
    public config?: RecursivePartial<SynConfig>
  ) {
    this.config = merge(config, defaultConfig());
  }

  get myPubKey() {
    return this.client.cellClient.cell.cell_id[1];
  }

  async fetchAllWorkspaces() {
    const workspaces = await this.client.getAllWorkspaces();

    this.knownWorkspaces.update(w => {
      for (const record of workspaces) {
        const entryHash = (record.signed_action.hashed.content as Create)
          .entry_hash;
        const workspace = decode(
          (record.entry as any).Present.entry
        ) as Workspace;
        w.put(entryHash, workspace);
      }

      return w;
    });

    return derived(this.knownWorkspaces, i => i);
  }

  async fetchAllCommits() {
    const commits = await this.client.getAllCommits();

    this.knownCommits.update(c => {
      for (const record of commits) {
        const entryHash = (record.signed_action.hashed.content as Create)
          .entry_hash;
        const commit = decode((record.entry as any).Present.entry) as Commit;
        c.put(entryHash, commit);
      }

      return c;
    });

    return derived(this.knownCommits, i => i);
  }

  async joinWorkspace<G extends SynGrammar<any, any>>(
    workspaceHash: EntryHash,
    grammar: G
  ) {
    return WorkspaceStore.joinWorkspace(
      this.client,
      grammar,
      this.config as SynConfig,
      workspaceHash
    );
  }

  async createWorkspace(
    workspace: Workspace,
    initialTipHash: EntryHash
  ): Promise<EntryHash> {
    const workspaceRecord = await this.client.createWorkspace({
      workspace,
      initial_tip_hash: initialTipHash,
    });
    const entryHash = (workspaceRecord.signed_action.hashed.content as Create)
      .entry_hash;
    this.knownWorkspaces.update(w => {
      const workspace = decode(
        (workspaceRecord.entry as any).Present.entry
      ) as Workspace;
      w.put(entryHash, workspace);

      return w;
    });

    return entryHash;
  }

  async createRootCommit<G extends SynGrammar<any, any>>(
    grammar: G,
    meta: any
  ): Promise<[EntryHash, Commit]> {
    let doc: Doc<any> = init();

    doc = change(doc, d => grammar.initState(d));

    if (meta) {
      meta = encode(meta);
    }

    const commit: Commit = {
      state: save(doc),
      authors: [this.myPubKey],
      meta,
      previous_commit_hashes: [],
      witnesses: [],
    };

    const record = await this.client.createCommit(commit);
    const entryHash = (record.signed_action.hashed.content as Create)
      .entry_hash;

    this.knownCommits.update(c => {
      c.put(entryHash, commit);
      return c;
    });

    return [entryHash, commit];
  }
}