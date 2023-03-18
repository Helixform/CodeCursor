// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from "vscode";
import fetch from "node-fetch";
import path = require("path");

async function generateCode(
    prompt: string,
    editor: vscode.TextEditor,
    cancellationToken: vscode.CancellationToken
): Promise<void> {
    // Current file path.
    const filePath = editor.document.uri.fsPath;

    const selection = editor.selection;
    const selectionText = editor.document.getText(selection);
    let currentCursor = editor.selection.active;

    const precedingCode = editor.document.getText(
        new vscode.Range(
            new vscode.Position(0, 0),
            new vscode.Position(selection.start.line, selection.start.character)
        )
    );
    const lastLine = editor.document.lineAt(editor.document.lineCount - 1);
    const suffixCode = editor.document.getText(
        new vscode.Range(
            new vscode.Position(selection.end.line, selection.end.character),
            new vscode.Position(
                lastLine.lineNumber,
                lastLine.range.end.character
            )
        )
    );

    // Split the code into chunks of 20 line blocks.
    function splitCodeIntoBlocks(code: string) {
        const lines = code.split("\n");
        const blocks = [];
        let currentBlock = [];
        for (const line of lines) {
            currentBlock.push(line);
            if (currentBlock.length >= 20) {
                blocks.push(currentBlock.join("\n"));
                currentBlock = [];
            }
        }
        if (currentBlock.length > 0) {
            blocks.push(currentBlock.join("\n"));
        }
        return blocks;
    }

    const requestBody = JSON.stringify({
        userRequest: {
            message: prompt,
            currentRootPath: path.dirname(filePath),
            currentFileName: filePath,
            currentFileContents: editor.document.getText(),
            precedingCode: splitCodeIntoBlocks(precedingCode),
            currentSelection: selectionText,
            suffixCode: splitCodeIntoBlocks(suffixCode),
            copilotCodeBlocks: [],
            customCodeBlocks: [],
            codeBlockIdentifiers: [],
            msgType: selectionText.length > 0 ? "edit" : "generate",
            maxOrigLine: null,
        },
        userMessages: [],
        botMessages: [],
        contextType: "copilot",
        rootPath: vscode.workspace.workspaceFolders?.[0].uri.fsPath,
    });

    const abortController = new AbortController();
    cancellationToken.onCancellationRequested(() => abortController.abort());
    const resp = await fetch("https://aicursor.com/conversation", {
        method: "POST",
        headers: {
            ["authority"]: "aicursor.com",
            ["accept"]: "*/*",
            ["content-type"]: "application/json",
            ["user-agent"]:
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.1.0 Chrome/108.0.5359.62 Electron/22.0.0 Safari/537.36",
        },
        body: requestBody,
        signal: abortController.signal,
    });

    const body = resp.body;
    if (!body) {
        console.error("error");
        return;
    }

    const promiseResolvers = {} as typeof Promise<void> extends new (
        executor: infer E
    ) => any
        ? E extends (resolve: infer R, reject: infer J) => any
            ? { resolve: R; reject: J }
            : never
        : never;
    const promise: Promise<void> = new Promise((resolve, reject) => {
        promiseResolvers.resolve = resolve;
        promiseResolvers.reject = reject;
    });

    const insertionQueue = [] as string[];
    let insertionQueueIsProcessing = false;
    function processQueue() {
        if (insertionQueueIsProcessing || abortController.signal.aborted) {
            return;
        }
        insertionQueueIsProcessing = true;
        const currentToken = insertionQueue.shift();
        if (!currentToken) {
            insertionQueueIsProcessing = false;
            return;
        }
        editor
            .edit((editBuilder) => {
                editBuilder.insert(currentCursor, currentToken);
            })
            .then(() => {
                const tokenLines = currentToken.split("\n");
                currentCursor = currentCursor.translate(
                    tokenLines.length - 1,
                    tokenLines[tokenLines.length - 1].length
                );
                editor.selection = new vscode.Selection(
                    currentCursor,
                    currentCursor
                );
                insertionQueueIsProcessing = false;
                processQueue();
            });
    }

    // Handle the SSE stream.
    let messageStarted = false;
    let messageEnded = false;
    body.on("data", (chunk: Buffer) => {
        if (messageEnded) {
            return;
        }

        const lines = chunk.toString().split("\n");
        for (const line of lines) {
            if (!line.startsWith('data: "')) {
                continue;
            }
            const data = JSON.parse(line.slice("data: ".length)) as string;
            if (data === "<|BEGIN_message|>") {
                messageStarted = true;
                continue;
            } else if (data === "<|END_message|>") {
                messageEnded = true;
                promiseResolvers.resolve();
                break;
            }

            if (messageStarted) {
                insertionQueue.push(data);
                processQueue();
            }
        }
    });

    return promise;
}

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
                                return generateCode(value, editor, token);
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
