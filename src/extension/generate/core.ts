import * as vscode from "vscode";

import { ResultStream } from "./resultStream";
import {
    generateCode as rustGenerateCode,
    ISelectionRange,
    IPosition,
} from "@crates/cursor-core";

export class SelectionRange implements ISelectionRange {
    private _start: Position;
    private _end: Position;

    constructor(selection: vscode.Selection) {
        this._start = new Position(
            selection.start.line,
            selection.start.character
        );
        this._end = new Position(selection.end.line, selection.end.character);
    }

    get start(): Position {
        return this._start;
    }

    get end(): Position {
        return this._end;
    }
}

export class Position implements IPosition {
    private _line: number;
    private _character: number;

    constructor(line: number, character: number) {
        this._line = line;
        this._character = character;
    }

    get line(): number {
        return this._line;
    }

    get character(): number {
        return this._character;
    }
}

export async function generateCode(
    prompt: string,
    document: vscode.TextDocument,
    selectionRange: SelectionRange,
    cursor: Position,
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

    await rustGenerateCode({
        prompt,
        documentText,
        filePath,
        workspaceDirectory,
        selectionRange,
        resultStream,
        abortSignal,
        cursor,
        languageId: document.languageId,
    });
}
