import * as vscode from "vscode";
import { generateCode } from "./generate/generate";
import {
    Scratchpad,
    getScratchpadManager,
    createUri as createScratchpadUri,
} from "./scratchpad";

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
    // The command has been defined in the package.json file
    // Now provide the implementation of the command with registerCommand
    // The commandId parameter must match the command field in package.json
    context.subscriptions.push(
        vscode.commands.registerCommand("aicursor.generateCode", () => {
            vscode.window
                .showInputBox({
                    placeHolder: "Instructions for code to generate...",
                })
                .then((value) => {
                    if (!value) {
                        return;
                    }

                    // Get the current editor.
                    const editor = vscode.window.activeTextEditor;
                    if (!editor) {
                        return;
                    }

                    const selection = editor.selection;
                    const selectionText = editor.document.getText(selection);

                    const scratchpad = new Scratchpad(selectionText);
                    vscode.commands.executeCommand(
                        "vscode.diff",
                        createScratchpadUri(scratchpad.id, true),
                        createScratchpadUri(scratchpad.id),
                        null,
                        {
                            viewColumn: vscode.ViewColumn.Beside,
                            preview: true,
                        } as vscode.TextDocumentShowOptions
                    );

                    vscode.window.withProgress(
                        {
                            location: vscode.ProgressLocation.Window,
                            title: "Generating code...",
                            cancellable: true,
                        },
                        (_progress, token) => {
                            return generateCode(
                                value,
                                editor,
                                token,
                                scratchpad
                            );
                        }
                    );
                });
        })
    );

    context.subscriptions.push(
        getScratchpadManager().registerTextDocumentContentProvider()
    );
}

// This method is called when your extension is deactivated
export function deactivate() {}
