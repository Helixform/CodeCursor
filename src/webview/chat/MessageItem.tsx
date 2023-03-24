import * as React from "react";

export interface MessageItemModel {
    id: string;
    contents: string;
    isReply?: boolean;
}

export interface MessageItemProps {
    model: MessageItemModel;
}

export function MessageItem(props: MessageItemProps) {
    const { model } = props;
    const { contents, isReply } = model;
    return (
        <div className="chat-msg">
            {isReply ? <div className="chat-msg-reply-bg" /> : null}
            <div className="chat-msg-contents">{contents}</div>
        </div>
    );
}
