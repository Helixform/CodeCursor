import * as vscode from "vscode";

import { ResultStream } from "../generate/resultStream";
import { SelectionRange } from "../generate/core";
import { getCustomModelConfiguration } from "../utils";
import {
    chat as rustChat,
    resetChat as rustResetChat,
} from "@crates/cursor-core";

let isProcessing = false;

export async function chat(
    prompt: string,
    msgType: string,
    document: vscode.TextDocument,
    selectionRange: SelectionRange,
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

    const customModelConfig = getCustomModelConfiguration();

    try {
        isProcessing = true;
        await rustChat({
            prompt,
            msgType,
            documentText,
            filePath,
            workspaceDirectory,
            selectionRange,
            resultStream,
            abortSignal,
            apiKey: customModelConfig?.openaiAPIKey || null,
            gptModel: customModelConfig?.model || null,
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
