
export default function LayerTree({ show } : { show: boolean }) {
    return (
        show && <div className="flex">
            <ul className="menu p-4 w-60 h-full bg-base-200 text-base-content">
                {/* Sidebar content here */}
                <li><a>Layer 1</a></li>
                <li><a>Layer 2</a></li>
            </ul>
        </div>
    )
}