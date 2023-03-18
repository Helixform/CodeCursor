import * as vscode from "vscode";
import { generateCode } from "./generate/generate";
import { ResultStream } from "./generate/result-stream";

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
    // The command has been defined in the package.json file
    // Now provide the implementation of the command with registerCommand
    // The commandId parameter must match the command field in package.json
    let disposable = vscode.commands.registerCommand(
        "aicursor.generateCode",
        () => {
            vscode.window
                .showInputBox({
                    placeHolder: "Instructions for code to generate...",
                })
                .then((value) => {
                    // Get the current editor.
                    const editor = vscode.window.activeTextEditor;
                    if (!editor) {
                        return;
                    }
                    if (value) {
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
                                    new ResultStream<String>()
                                );
                            }
                        );
                    }
                });
        }
    );

    context.subscriptions.push(disposable);
}

// This method is called when your extension is deactivated
export function deactivate() {}
