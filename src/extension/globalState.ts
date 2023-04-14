import { Memento } from "vscode";
import { GenerateSession } from "./generate";

interface GlobalState {
    activeSession: GenerateSession | null;
    storage: Memento | null;
}

const globalState: GlobalState = {
    activeSession: null,
    storage: null,
};

export function getGlobalState() {
    return globalState;
}
