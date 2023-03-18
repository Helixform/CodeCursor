import * as vscode from "vscode";

import { ResultStream } from "./generate/result-stream";

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

    getScratchpad(docId: string): Scratchpad | undefined {
        return this.documents.get(docId);
    }

    notifyDocumentChange(docId: string) {
        this._didChangeEventEmitter.fire(createUri(docId));
    }

    provideTextDocumentContent(
        uri: vscode.Uri,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<string> {
        console.log(uri);
        const docId = uri.query;
        const doc = this.documents.get(docId);
        if (!doc) {
            return null;
        }
        if (uri.path === "orig") {
            return doc.originalContents;
        }
        return doc.contents;
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

    constructor(originalContents: string) {
        this.id = scratchpadManager.addScratchpad(this);
        this.originalContents = originalContents;
        this.contents = "";
    }

    write(text: string) {
        this.contents += text;
        scratchpadManager.notifyDocumentChange(this.id);
    }

    end(): void {}
}
