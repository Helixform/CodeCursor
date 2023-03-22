import * as vscode from "vscode";
import path = require("path");
import { ResultStream } from "./result-stream";
import {
    generateCode as rustGenerateCode,
    IGenerateInput,
    IResultStream,
    ISelectionRange,
} from "../../crates/cursor-core/pkg";

class GenerateInput implements IGenerateInput {
    private _prompt: string;
    private _documentText: string;
    private _filePath: string;
    private _workspaceDirectory: string | null;
    private _selectionRange: ISelectionRange;
    private _resultStream: IResultStream;

    constructor(
        prompt: string,
        documentText: string,
        filePath: string,
        workspaceDirectory: string | null,
        selectionRange: ISelectionRange,
        resultStream: IResultStream
    ) {
        this._prompt = prompt;
        this._documentText = documentText;
        this._filePath = filePath;
        this._workspaceDirectory = workspaceDirectory;
        this._selectionRange = selectionRange;
        this._resultStream = resultStream;
    }

    get prompt(): string {
        return this._prompt;
    }

    get documentText(): string {
        return this._documentText;
    }

    get filePath(): string {
        return this._filePath;
    }

    get workspaceDirectory(): string | null {
        return this._workspaceDirectory;
    }

    get selectionRange(): ISelectionRange {
        return this._selectionRange;
    }

    get resultStream(): IResultStream {
        return this._resultStream;
    }
}

class SelectionRange implements ISelectionRange {
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
    editor: vscode.TextEditor,
    cancellationToken: vscode.CancellationToken,
    resultStream: ResultStream<String>
): Promise<void> {
    const document = editor.document;
    const filePath = document.uri.fsPath;
    const workspaceDirectory =
        vscode.workspace.workspaceFolders?.[0].uri.fsPath ?? null;
    const selection = editor.selection;
    const text = document.getText();
    const selectionStartOffset = document.offsetAt(selection.start);
    const selectionEndOffset = document.offsetAt(selection.end);
    const selectionRange = new SelectionRange(
        selectionStartOffset,
        selectionEndOffset - selectionStartOffset
    );

    await rustGenerateCode(
        new GenerateInput(
            prompt,
            text,
            filePath,
            workspaceDirectory,
            selectionRange,
            resultStream
        )
    );
}
