import { writable, readable, derived } from 'svelte/store';

export const content = writable({title:"", body:""});

export const pendingDeltas = writable([]);

export const folks = writable({});

export const connection = writable();

export const scribeStr = writable("");

export const recordedChanges = writable([]);

export const requestedChanges = writable([]);

export const nextIndex = writable(0)
