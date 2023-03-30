import * as vscode from "vscode";

import { ResultStream } from "./resultStream";
import { getCustomModelConfiguration } from "../utils";
import {
    generateCode as rustGenerateCode,
    ISelectionRange,
} from "@crates/cursor-core";

export class SelectionRange implements ISelectionRange {
    private _offset: number;
    private _length: number;

    constructor(offset: number, length: number) {
        this._offset = offset;
        this._length = length;
    }

    get offset(): number {
        return this._offset;
    }

    get length(): number {
        return this._length;
    }
}

export async function generateCode(
    prompt: string,
    document: vscode.TextDocument,
    selectionRange: SelectionRange,
    cancellationToken: vscode.CancellationToken,
    resultStream: ResultStream<String>
): Promise<void> {
    const filePath = document.uri.fsPath;
    const workspaceDirectory =
        vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath ?? null;
    const documentText = document.getText();

    const abortController = new AbortController();
    cancellationToken.onCancellationRequested(() => {
        abortController.abort();
    });
    const { signal: abortSignal } = abortController;

    const customModelConfig = getCustomModelConfiguration();

    await rustGenerateCode({
        prompt,
        documentText,
        filePath,
        workspaceDirectory,
        selectionRange,
        resultStream,
        abortSignal,
        apiKey: customModelConfig?.openaiAPIKey || null,
        gptModel: customModelConfig?.model || null,
    });
}
