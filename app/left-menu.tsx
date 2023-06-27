export enum LeftMenuItems {
    Layers,
}

export default function LeftMenu({ selectedItem, updateSelectedItem }: { selectedItem: LeftMenuItems | undefined, updateSelectedItem: (item: LeftMenuItems) => void }) {
    return (
        <ul className="menu bg-base-100">
            <li className="active">
                <a onClick={() => { updateSelectedItem(LeftMenuItems.Layers) }} className={selectedItem == LeftMenuItems.Layers ? "active" : ""}>
                    <svg viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" width="19" height="19"><path d="M7.5 1.5l.197-.46a.5.5 0 00-.394 0l.197.46zm-7 3l-.197-.46a.5.5 0 000 .92L.5 4.5zm7 3l-.197.46a.5.5 0 00.394 0L7.5 7.5zm7-3l.197.46a.5.5 0 000-.92l-.197.46zm-7 6l-.197.46.197.084.197-.084-.197-.46zm0 3l-.197.46.197.084.197-.084-.197-.46zM7.303 1.04l-7 3 .394.92 7-3-.394-.92zm-7 3.92l7 3 .394-.92-7-3-.394.92zm7.394 3l7-3-.394-.92-7 3 .394.92zm7-3.92l-7-3-.394.92 7 3 .394-.92zM.303 7.96l7 3 .394-.92-7-3-.394.92zm7.394 3l7-3-.394-.92-7 3 .394.92zm-7.394 0l7 3 .394-.92-7-3-.394.92zm7.394 3l7-3-.394-.92-7 3 .394.92z" fill="currentColor"></path></svg>
                </a>
            </li>
        </ul>
    )
}