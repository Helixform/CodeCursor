import * as vscode from "vscode";

import { ResultStream } from "../generate/resultStream";
import { SelectionRange } from "../generate/core";
import { chat as rustChat } from "@crates/cursor-core";

let isProcessing = false;

export async function chat(
    prompt: string,
    document: vscode.TextDocument,
    selectionRange: SelectionRange,
    cancellationToken: vscode.CancellationToken,
    resultStream: ResultStream<String>
): Promise<void> {
    if (isProcessing) {
        throw new Error("A chat session is in-flight");
    }

    const filePath = document.uri.fsPath;
    const workspaceDirectory =
        vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath ?? null;
    const documentText = document.getText();

    const abortController = new AbortController();
    cancellationToken.onCancellationRequested(() => {
        abortController.abort();
    });
    const { signal: abortSignal } = abortController;

    try {
        isProcessing = true;
        await rustChat({
            prompt,
            documentText,
            filePath,
            workspaceDirectory,
            selectionRange,
            resultStream,
            abortSignal,
        });
    } finally {
        isProcessing = false;
    }
}
