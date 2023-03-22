import * as vscode from "vscode";

import { Scratchpad } from "./scratchpad";
import { generateCode } from "./core";

export class GenerateSession {
    #prompt: string;
    #selection: vscode.Selection;
    #document: vscode.TextDocument;
    #scratchpad: Scratchpad | null;
    #errorOccurred = false;
    #statusBarItem: vscode.StatusBarItem | null = null;

    constructor(prompt: string, editor: vscode.TextEditor) {
        const { document, selection } = editor;
        const selectionText = document.getText(selection);

        this.#prompt = prompt;
        this.#selection = selection;
        this.#document = document;
        this.#scratchpad = new Scratchpad(selectionText);
    }

    dispose() {
        this.hideResult();

        this.#scratchpad?.dispose();
        this.#scratchpad = null;

        this.#statusBarItem?.dispose();
        this.#statusBarItem = null;
    }

    start() {
        const scratchpad = this.#scratchpad;
        if (!scratchpad) {
            throw new Error("The session is disposed");
        }

        vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: "Generating code...",
                cancellable: true,
            },
            async (_progress, token) => {
                try {
                    await generateCode(
                        this.#prompt,
                        this.#document,
                        this.#selection,
                        token,
                        scratchpad
                    );
                    this.#showGenerationDecisionMessage();
                } catch (e) {
                    console.error(e);
                    this.#errorOccurred = true;
                    this.#showErrorDecisionMessage();
                    return;
                }
            }
        );
    }

    showResult() {
        const scratchpad = this.#scratchpad;
        if (!scratchpad) {
            return;
        }

        if (!this.#getOpenedResultTab()) {
            vscode.commands.executeCommand(
                "vscode.diff",
                scratchpad.uriForOriginalContents,
                scratchpad.uri,
                null,
                {
                    viewColumn: vscode.ViewColumn.Beside,
                    preview: true,
                } as vscode.TextDocumentShowOptions
            );
        }

        if (this.#errorOccurred) {
            this.#showErrorDecisionMessage();
        } else if (this.#scratchpad?.ended) {
            this.#showGenerationDecisionMessage();
        }
    }

    hideResult() {
        const openedResultTab = this.#getOpenedResultTab();
        if (!openedResultTab) {
            return;
        }

        vscode.window.tabGroups.close(openedResultTab);
    }

    #retry() {
        this.#statusBarItem?.dispose();
        this.#statusBarItem = null;

        this.#scratchpad?.reset();

        this.#errorOccurred = false;

        this.start();
    }

    #getOpenedResultTab(): vscode.Tab | null {
        const scratchpad = this.#scratchpad;
        if (!scratchpad) {
            return null;
        }

        const thisUriString = scratchpad.uri.toString();
        const tabGroups = vscode.window.tabGroups;
        for (const tabGroup of tabGroups.all) {
            for (const tab of tabGroup.tabs) {
                const tabInput = tab.input;
                if (!(tabInput instanceof vscode.TabInputTextDiff)) {
                    continue;
                }
                if (tabInput.modified.toString() == thisUriString) {
                    return tab;
                }
            }
        }

        return null;
    }

    #getEditor(): vscode.TextEditor | null {
        const documentUriString = this.#document.uri.toString();
        for (const editor of vscode.window.visibleTextEditors) {
            if (editor.document.uri.toString() === documentUriString) {
                return editor;
            }
        }
        return null;
    }

    async #showGenerationDecisionMessage() {
        if (!this.#statusBarItem) {
            const statusBarItem = vscode.window.createStatusBarItem(
                vscode.StatusBarAlignment.Left
            );
            statusBarItem.text = "$(check) Code Cursor";
            statusBarItem.tooltip = "View Code Generation Result";
            statusBarItem.color = new vscode.ThemeColor("button.foreground");
            statusBarItem.command = "aicursor.showLastResult";
            statusBarItem.show();
            this.#statusBarItem = statusBarItem;
        }

        const pick = await vscode.window.showInformationMessage(
            "Code generation is done.",
            {
                detail: "What do you want to do with the result?",
            },
            "Accept",
            "Reject"
        );
        if (pick === "Accept") {
            this.#applyChanges();
        } else if (pick === "Reject") {
            this.dispose();
        }
    }

    async #showErrorDecisionMessage() {
        if (!this.#statusBarItem) {
            const statusBarItem = vscode.window.createStatusBarItem(
                vscode.StatusBarAlignment.Left
            );
            statusBarItem.text = "$(error) Code Cursor";
            statusBarItem.tooltip = "Retry";
            statusBarItem.backgroundColor = new vscode.ThemeColor(
                "statusBarItem.errorBackground"
            );
            statusBarItem.color = new vscode.ThemeColor("button.foreground");
            statusBarItem.command = "aicursor.showLastResult";
            statusBarItem.show();
            this.#statusBarItem = statusBarItem;
        }

        const pick = await vscode.window.showInformationMessage(
            "Failed to perform code generation.",
            "Retry",
            "Cancel"
        );
        if (pick === "Retry") {
            this.#retry();
        } else if (pick === "Cancel") {
            this.dispose();
        }
    }

    #applyChanges() {
        const scratchpad = this.#scratchpad;
        if (!scratchpad) {
            return;
        }

        const editor = this.#getEditor();
        if (!editor) {
            vscode.window.showWarningMessage(
                "Need to activate the original text buffer before applying changes."
            );
            return;
        }

        // TODO: reconcile with the modified document.
        editor
            .edit((editBuilder) => {
                editBuilder.replace(this.#selection, scratchpad.contents);
            })
            .then(() => {
                // Dispose self after changes are applied.
                this.dispose();
            });
    }
}
