import {
    IChatViewService,
    CHAT_VIEW_SERVICE_NAME,
} from "../../common/chatService";
import { MessageItemModel } from "../../common/chatService/model";

export class ChatViewServiceImpl implements IChatViewService {
    setIsReadyAction: ((isReady: boolean) => void) | null = null;
    setHasSelectionAction: ((hasSelection: boolean) => void) | null = null;
    addMessageAction: ((msg: MessageItemModel) => void) | null = null;
    updateMessageAction: ((msg: MessageItemModel) => void) | null = null;

    get name(): string {
        return CHAT_VIEW_SERVICE_NAME;
    }

    async setIsBusy(isBusy: boolean): Promise<void> {
        this.setIsReadyAction?.call(null, isBusy);
    }

    async setHasSelection(hasSelection: boolean): Promise<void> {
        this.setHasSelectionAction?.call(null, hasSelection);
    }

    async addMessage(msg: MessageItemModel): Promise<void> {
        this.addMessageAction?.call(null, msg);
    }

    async updateMessage(msg: MessageItemModel): Promise<void> {
        this.updateMessageAction?.call(null, msg);
    }
}
