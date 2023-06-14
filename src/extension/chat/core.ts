import * as vscode from "vscode";

import { ResultStream } from "../generate/resultStream";
import { Position, SelectionRange } from "../generate/core";
import {
    chat as rustChat,
    resetChat as rustResetChat,
} from "@crates/cursor-core";

let isProcessing = false;

export async function chat(
    prompt: string,
    document: vscode.TextDocument,
    selection: vscode.Selection,
    abortSignal: AbortSignal,
    resultStream: ResultStream<String>
): Promise<void> {
    if (isProcessing) {
        throw new Error("A chat session is in-flight");
    }

    const filePath = document.uri.fsPath;
    const workspaceDirectory =
        vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath ?? null;
    const documentText = document.getText();

    try {
        isProcessing = true;
        await rustChat({
            prompt,
            documentText,
            filePath,
            workspaceDirectory,
            selectionRange: new SelectionRange(selection),
            resultStream,
            abortSignal,
            cursor: new Position(
                selection.active.line,
                selection.active.character
            ),
            languageId: document.languageId,
        });
    } finally {
        isProcessing = false;
    }
}

export function resetChat() {
    if (isProcessing) {
        throw new Error("Cannot reset the chat session while it's in-flight");
    }

    rustResetChat();
}
