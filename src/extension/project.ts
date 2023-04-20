import * as vscode from "vscode";
import { generateProject } from "@crates/cursor-core";

export async function handleGenerateProjectCommand() {
    const input = await vscode.window.showInputBox({
        title: "Generate A New Project",
        placeHolder: "Instructions for project to generate...",
    });
    if (!input) {
        return;
    }
    await generateProject(input);
}
