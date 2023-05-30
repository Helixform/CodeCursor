import * as vscode from "vscode";
import { diff_match_patch } from "diff-match-patch";

import { Scratchpad } from "./scratchpad";
import { generateCode, Position, SelectionRange } from "./core";
import { getOpenedTab } from "../utils";

export class GenerateSession {
    #prompt: string;
    #selectionRange: SelectionRange;
    #cursor: Position;
    #document: vscode.TextDocument;
    #documentSnapshot: string;
    #scratchpad: Scratchpad | null;
    #errorOccurred = false;
    #statusBarItem: vscode.StatusBarItem | null = null;
    #disposeEvent = new vscode.EventEmitter<void>();

    constructor(prompt: string, editor: vscode.TextEditor) {
        const { document, selection } = editor;
        const documentSnapshot = document.getText();
        const selectionText = document.getText(selection);

        this.#prompt = prompt;
        this.#selectionRange = new SelectionRange(selection);
        this.#cursor = new Position(
            selection.active.line,
            selection.active.character
        );
        this.#document = document;
        this.#documentSnapshot = documentSnapshot;
        this.#scratchpad = new Scratchpad(selectionText);
    }

    get onDidDispose(): vscode.Event<void> {
        return this.#disposeEvent.event;
    }

    dispose() {
        this.hideResult();

        this.#scratchpad?.dispose();
        this.#scratchpad = null;

        this.#statusBarItem?.dispose();
        this.#statusBarItem = null;

        this.#disposeEvent.fire();
        this.#disposeEvent.dispose();
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
                        this.#selectionRange,
                        this.#cursor,
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

        if (!getOpenedTab(scratchpad.uri)) {
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
        const scratchpad = this.#scratchpad;
        if (!scratchpad) {
            return;
        }

        const openedResultTab = getOpenedTab(scratchpad.uri);
        if (!openedResultTab) {
            return;
        }

        vscode.window.tabGroups.close(openedResultTab);
    }

    applyChanges() {
        if (this.#scratchpad?.ended) {
            this.#applyChanges();
        }
    }

    #retry() {
        this.#statusBarItem?.dispose();
        this.#statusBarItem = null;

        this.#scratchpad?.reset();

        this.#errorOccurred = false;

        this.start();
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
            statusBarItem.text = "$(check) CodeCursor";
            statusBarItem.tooltip = "View Code Generation Result";
            statusBarItem.color = new vscode.ThemeColor("button.foreground");
            statusBarItem.command = "aicursor.showLastResult";
            statusBarItem.show();
            this.#statusBarItem = statusBarItem;
        }

        const pick = await vscode.window.showInformationMessage(
            "Code generation is done.",
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
            statusBarItem.text = "$(error) CodeCursor";
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
        const { document } = editor;

        // Use DMP to reconcile the generated contents with the modified document.
        const selectionRange = this.#selectionRange;
        const originalContents = this.#documentSnapshot;
        const startOffset = document.offsetAt(
            new vscode.Position(
                selectionRange.start.line,
                selectionRange.start.character
            )
        );
        const endOffset = document.offsetAt(
            new vscode.Position(
                selectionRange.end.line,
                selectionRange.end.character
            )
        );
        const patchedOriginalContents =
            originalContents.substring(0, startOffset) +
            scratchpad.contents +
            originalContents.substring(endOffset);

        const dmp = new diff_match_patch();
        const diff = dmp.diff_main(
            originalContents,
            patchedOriginalContents,
            true
        );
        const patch = dmp.patch_make(
            originalContents,
            patchedOriginalContents,
            diff
        );

        const currentContents = document.getText() || "";
        const patchApplyResults = dmp.patch_apply(patch, currentContents);

        // Check whether we can apply the changes.
        const hasPatchFailures = patchApplyResults[1].filter(
            (ok) => !ok
        ).length;
        if (hasPatchFailures) {
            vscode.window.showWarningMessage(
                "The document has changed, cannot apply the changes automatically now. You " +
                    "can still copy the generated contents back manually."
            );
            return;
        }

        // We got a complete patched contents here, it's fine to fully replace it to the
        // document buffer. This is an asynchronous process, however, VSCode will make
        // sure that there will be no concurrent changes.
        const finalContents = patchApplyResults[0];
        editor
            .edit((editBuilder) => {
                const rangeStart = document.positionAt(0);
                const rangeEnd = document.positionAt(currentContents.length);
                editBuilder.replace(
                    new vscode.Range(rangeStart, rangeEnd),
                    finalContents
                );
            })
            .then((success) => {
                if (!success) {
                    // Concurrent modifications did happen, just let user try again.
                    vscode.window.showWarningMessage(
                        "Failed to apply the changes, maybe there are concurrent " +
                            "modifications. You can try again later."
                    );
                    return;
                }

                // Dispose self after changes are applied.
                this.dispose();
            });
    }
}
