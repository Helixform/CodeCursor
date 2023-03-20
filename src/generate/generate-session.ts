import * as vscode from "vscode";

import { Scratchpad } from "./scratchpad";
import { generateCode } from "./generate";

export class GenerateSession {
    #prompt: string;
    #selection: vscode.Selection;
    #editor: vscode.TextEditor;
    #scratchpad: Scratchpad | null;
    #errorOccurred = false;
    #statusBarItem: vscode.StatusBarItem | null = null;

    constructor(
        prompt: string,
        selection: vscode.Selection,
        editor: vscode.TextEditor
    ) {
        const selectionText = editor.document.getText(selection);

        this.#prompt = prompt;
        this.#selection = selection;
        this.#editor = editor;
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
                        this.#editor,
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
        if (!this.#getOpenedResultTab()) {
            this.#scratchpad?.showInDiffView();
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
        for (let tabGroup of tabGroups.all) {
            for (let tab of tabGroup.tabs) {
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

        this.#editor
            .edit((editBuilder) => {
                editBuilder.replace(this.#selection, scratchpad.contents);
            })
            .then(() => {
                // Dispose self after changes are applied.
                this.dispose();
            });
    }
}
