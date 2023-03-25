export interface MessageItemModel {
    id: string;
    contents: string;
    isReply?: boolean;
    isFinished?: boolean;
}
