

export default function ModelCard({
    img,
    title,
    description,
    tags = [],
    isNew = false,
}: {
    img: string,
    title: string,
    description: string,
    tags: string[],
    isNew: boolean,
}) {
    return (
        <div className="card w-64 bg-base-100 shadow-xl h-full">
            <figure><img src={img} alt="model" /></figure>
            <div className="card-body">
                <h2 className="card-title">
                    {title}
                    {isNew && <div className="badge badge-secondary">NEW</div>}
                </h2>
                <p>{description}</p>
                <div className="card-actions justify-end">
                    {
                        tags?.map((tag, index) =>
                            (<div key={index} className="badge badge-outline">{tag}</div>)
                        )
                    }
                </div>
            </div>
        </div>
    )
}