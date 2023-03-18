import * as vscode from "vscode";

class ScratchpadDocument {
    id: string;
    contents: string;

    constructor(id: string) {
        this.id = id;
        this.contents = "";
    }

    appendText(text: string) {
        this.contents += text;
        getScratchpadManager().notifyDocumentChange(this.id);
    }
}

const URI_SCHEME = "ccsp";
export function createUri(docId: string): vscode.Uri {
    return vscode.Uri.parse(`${URI_SCHEME}:cursor-reply?${docId}`);
}

export class ScratchpadManager implements vscode.TextDocumentContentProvider {
    onDidChange?: vscode.Event<vscode.Uri> | undefined;
    documents: Map<string, ScratchpadDocument> = new Map();
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

    addScratchpad(contents?: string): string {
        ++this.currentId;

        const docId = `${this.currentId}`;
        const doc = new ScratchpadDocument(docId);
        this.documents.set(docId, doc);

        if (contents) {
            doc.contents = contents;
        }

        return docId;
    }

    getScratchpadDocument(docId: string): ScratchpadDocument | undefined {
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
        return doc.contents;
    }
}

const scratchpadManager = new ScratchpadManager();
export function getScratchpadManager(): ScratchpadManager {
    return scratchpadManager;
}

export class Scratchpad {
    document: ScratchpadDocument;

    constructor(docId: string) {
        const document = scratchpadManager.getScratchpadDocument(docId);
        if (!document) {
            throw new Error(`Document with id "${docId}" not found`);
        }
        this.document = document;
    }

    async open(): Promise<void> {
        const doc = await vscode.workspace.openTextDocument(
            createUri(this.document.id)
        );
        await vscode.window.showTextDocument(doc, {
            viewColumn: vscode.ViewColumn.Beside,
            preserveFocus: true,
        });
    }

    close() {
        const uriString = createUri(this.document.id).toString();
        for (let editor of vscode.window.visibleTextEditors) {
            if (editor.document.uri.toString() === uriString) {
                editor.hide();
                return;
            }
        }
    }

    appendText(text: string) {
        this.document.appendText(text);
    }
}
