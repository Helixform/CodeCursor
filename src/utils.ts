import * as vscode from 'vscode';

export function getOpenedTab(uri: vscode.Uri): vscode.Tab | null {
    const targetUriString = uri.toString();
    const tabGroups = vscode.window.tabGroups;
    for (const tabGroup of tabGroups.all) {
        for (const tab of tabGroup.tabs) {
            const tabInput = tab.input;
            if (!(tabInput instanceof vscode.TabInputTextDiff)) {
                continue;
            }
            if (tabInput.modified.toString() === targetUriString) {
                return tab;
            }
        }
    }

    return null;
}
