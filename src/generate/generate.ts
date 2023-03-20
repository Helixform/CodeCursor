import * as vscode from "vscode";
import fetch from "node-fetch";
import path = require("path");
import { ResultStream } from "./result-stream";
import { v4 as uuidv4 } from "uuid";
import {
    BotMessage,
    BotMessageType,
    interruptedBotMessage,
} from "./bot-message";

const headers = {
    ["authority"]: "aicursor.com",
    ["accept"]: "*/*",
    ["content-type"]: "application/json",
    ["user-agent"]:
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.1.0 Chrome/108.0.5359.62 Electron/22.0.0 Safari/537.36",
};

export async function generateCode(
    prompt: string,
    editor: vscode.TextEditor,
    cancellationToken: vscode.CancellationToken,
    resultStream: ResultStream<String>
): Promise<void> {
    // Current file path.
    const filePath = editor.document.uri.fsPath;

    const selection = editor.selection;
    const selectionText = editor.document.getText(selection);

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
            if (line.length === 0) {
                continue;
            }
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

    const messageType =
        selectionText.length > 0
            ? BotMessageType.edit
            : BotMessageType.generate;

    const requestBody = {
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
            msgType: messageType,
            maxOrigLine: null,
        },
        userMessages: [],
        botMessages: [] as BotMessage[],
        contextType: "copilot",
        rootPath: vscode.workspace.workspaceFolders?.[0].uri.fsPath,
    };

    const abortController = new AbortController();
    cancellationToken.onCancellationRequested(() => abortController.abort());

    /// A Boolean value indicating whether the conversation is finished.
    let finished = false;
    // If the conversation was interrupted, we need to send a "continue" request.
    let interrupted = false;
    // Handle the SSE stream.
    let messageStarted = false;
    let firstNewlineDropped = false;

    let conversationId: string | null = null;
    // The last message received from the server.
    let previousMessage: string = "";
    let lastToken = "";

    let counter = 0;
    while (!finished) {
        if (interrupted) {
            counter++;
            // Generate an UUID as conversation ID.
            if (!conversationId) {
                conversationId = uuidv4();
            }
            const botMessage = interruptedBotMessage(
                messageType,
                conversationId,
                previousMessage,
                lastToken,
                filePath
            );
            requestBody.botMessages = [botMessage];
        }

        const resp = await fetch(
            `https://aicursor.com/${interrupted ? "continue" : "conversation"}`,
            {
                method: "POST",
                headers,
                body: JSON.stringify(requestBody),
                signal: abortController.signal,
            }
        );

        const body = resp.body;
        if (!body) {
            console.error("error");
            return;
        }

        // Reset the interrupted flag.
        interrupted = false;

        for await (const chunk of body) {
            const lines = chunk
                .toString()
                .split("\n")
                .filter((l) => l.length > 0);
            let messageEnded = false;
            for (const line of lines) {
                if (!line.startsWith('data: "')) {
                    continue;
                }
                // A string can be JSON to parse.
                let data = JSON.parse(line.slice("data: ".length)) as string;
                if (data === "<|BEGIN_message|>") {
                    messageStarted = true;
                    continue;
                } else if (data.includes("<|END_interrupt|>")) {
                    interrupted = true;
                    lastToken = data;
                    // `END_interrupt` is included in valid data,
                    // we cannot discard it.
                    data = data.replace("<|END_interrupt|>", "");
                } else if (data === "<|END_message|>") {
                    if (!interrupted) {
                        finished = true;
                    }
                    // We cannot exit the loop here because we're in a nested loop.
                    messageEnded = true;
                    break;
                }

                if (messageStarted) {
                    // Server may produce newlines at the head of response, we need
                    // to do this trick to ignore them in the final edit.
                    if (!firstNewlineDropped && data.trim().length === 0) {
                        firstNewlineDropped = true;
                        continue;
                    }
                    resultStream.write(data);
                    previousMessage += data;
                }
            }
            // If we've reached the end of the message, break out of the loop.
            if (messageEnded) {
                //break;
            }
        }
    }

    resultStream.end();
}
