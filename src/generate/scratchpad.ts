import * as vscode from "vscode";

import { ResultStream } from "./resultStream";

const URI_SCHEME = "ccsp";
export function createUri(docId: string, orig: boolean = false): vscode.Uri {
    return vscode.Uri.parse(`${URI_SCHEME}:${orig ? "orig" : "new"}?${docId}`);
}

export class ScratchpadManager implements vscode.TextDocumentContentProvider {
    onDidChange?: vscode.Event<vscode.Uri> | undefined;
    documents: Map<string, Scratchpad> = new Map();
    currentId = 0;
    _didChangeEventEmitter = new vscode.EventEmitter<vscode.Uri>();

    constructor() {
        this.onDidChange = this._didChangeEventEmitter.event;
    }

    registerTextDocumentContentProvider(): vscode.Disposable {
        return vscode.workspace.registerTextDocumentContentProvider(
            URI_SCHEME,
            this
        );
    }

    addScratchpad(scratchpad: Scratchpad): string {
        ++this.currentId;

        const docId = `${this.currentId}`;
        this.documents.set(docId, scratchpad);

        return docId;
    }

    removeScratchpad(scratchpad: Scratchpad) {
        this.documents.delete(scratchpad.id);
    }

    getScratchpad(docId: string): Scratchpad | undefined {
        return this.documents.get(docId);
    }

    notifyDocumentChange(docId: string) {
        this._didChangeEventEmitter.fire(createUri(docId));
    }

    provideTextDocumentContent(
        uri: vscode.Uri,
        _token: vscode.CancellationToken
    ): vscode.ProviderResult<string> {
        const docId = uri.query;
        const doc = this.documents.get(docId);
        if (!doc) {
            return null;
        }
        if (uri.path === "orig") {
            return doc.originalContents;
        }

        // Return the new contents.
        if (doc.ended) {
            return doc.contents;
        }
        // Progressively replace the original contents with the new contents.
        const origLines = doc.originalContents.split("\n");
        const newLines = doc.contents.split("\n");
        const intermediateLines = origLines.slice(newLines.length);
        intermediateLines.unshift(...newLines);
        return intermediateLines.join("\n");
    }
}

const scratchpadManager = new ScratchpadManager();
export function getScratchpadManager(): ScratchpadManager {
    return scratchpadManager;
}

export class Scratchpad implements ResultStream<string> {
    id: string;
    originalContents: string;
    contents: string;
    ended: boolean;
    #lastUpdateTime: number = 0;
    #deferredUpdateScheduled = false;

    constructor(originalContents: string) {
        this.id = scratchpadManager.addScratchpad(this);
        this.originalContents = originalContents;
        this.contents = "";
        this.ended = false;
    }

    dispose() {
        scratchpadManager.removeScratchpad(this);
    }

    get uri(): vscode.Uri {
        return createUri(this.id);
    }

    get uriForOriginalContents(): vscode.Uri {
        return createUri(this.id, true);
    }

    reset() {
        this.contents = "";
        this.ended = false;
        this.#notifyChanges();
    }

    write(text: string) {
        this.contents += text;
        this.#notifyChanges(true);
    }

    end(): void {
        this.ended = true;
        this.#notifyChanges();
    }

    #notifyChanges(throttle?: boolean) {
        const now = Date.now();
        if (throttle) {
            const elapseSinceLastUpdate = now - this.#lastUpdateTime;
            // TODO: add a setting item for the throttle time.
            if (elapseSinceLastUpdate < 500) {
                this.#scheduleDeferredUpdate(500 - elapseSinceLastUpdate);
                return;
            }
        }

        scratchpadManager.notifyDocumentChange(this.id);
        this.#lastUpdateTime = now;
    }

    #scheduleDeferredUpdate(delayMs: number) {
        if (this.#deferredUpdateScheduled) {
            return;
        }
        this.#deferredUpdateScheduled = true;
        setTimeout(() => {
            this.#deferredUpdateScheduled = false;
            this.#notifyChanges();
        }, delayMs);
    }
}
