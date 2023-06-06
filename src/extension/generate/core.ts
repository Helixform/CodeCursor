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
    private _startLine: number;
    private _startColumn: number;
    private _endLine: number;
    private _endColumn: number;

    constructor(offset: number, length: number, startLine: number, startColumn: number, endLine: number, endColumn: number) {
        this._offset = offset;
        this._length = length;
        this._startLine = startLine;
        this._startColumn = startColumn;
        this._endLine = endLine;
        this._endColumn = endColumn;
    }

    get offset(): number {
        return this._offset;
    }

    get length(): number {
        return this._length;
    }

    get startLine(): number {
        return this._startLine;
    }

    get startColumn(): number {
        return this._startColumn;
    }

    get endLine(): number {
        return this._endLine;
    }

    get endColumn(): number {
        return this._endColumn;
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
