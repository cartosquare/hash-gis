import { useEffect, useMemo } from "react";
import { useMap } from "react-leaflet";
import { useMapLayers } from "../context/maplayers-context";
import { MapSettings } from "../types";
import { createLeafletBounds } from "../utils";



export const MapLegend = () => {
    const map = useMap();
    const mapLayers = useMapLayers();

    useEffect(() => {
    }, [])

    const zoomToLayer = (index: number) => {
        map.fitBounds(createLeafletBounds(mapLayers.data.layers[index].bounds as number[]));
    }

    // Memoize the minimap so it's not affected by position changes
    const minimap = useMemo(
        () => (
            <div className="flex flex-col m-4 bg-base-300 opacity-90 shadow-md">
                {
                    mapLayers.data.layers.map((s: MapSettings, i: number) => {
                        const index = mapLayers.data.layers.length - i - 1;
                        return (
                            <div
                                key={index}
                                className="flex flex-row py-2">
                                <div className="flex">
                                    <input
                                        type="checkbox"
                                        defaultChecked
                                        onClick={() => { mapLayers.toggleLayer(index) }}
                                        className="toggle toggle-secondary">
                                    </input>
                                </div>

                                <div className="flex items-center px-2">
                                    <p className="text-center">
                                        {`${mapLayers.data.layers[index].path.replace(/^.*[\\\/]/, '')}`}
                                    </p>
                                </div>

                                <div className="flex grow justify-end">
                                    <button
                                        onClick={() => { mapLayers.refreshLayer(index) }}
                                        disabled={mapLayers.data.layers[index].geo_type == "raster"}
                                        className="btn btn-ghost btn-xs">
                                        <svg xmlns="http://www.w3.org/2000/svg" className="icon icon-tabler icon-tabler-edit w-5 h-5" viewBox="0 0 24 24" strokeWidth="1.5" stroke="#9e9e9e" fill="none" strokeLinecap="round" strokeLinejoin="round">
                                            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                                            <path d="M7 7h-1a2 2 0 0 0 -2 2v9a2 2 0 0 0 2 2h9a2 2 0 0 0 2 -2v-1" />
                                            <path d="M20.385 6.585a2.1 2.1 0 0 0 -2.97 -2.97l-8.415 8.385v3h3l8.385 -8.415z" />
                                            <path d="M16 5l3 3" />
                                        </svg>
                                    </button>
                                </div>
                                {/* <div className="flex"> */}

                                    <button
                                        onClick={() => { zoomToLayer(index) }}
                                        className="btn btn-ghost btn-xs">
                                        <svg xmlns="http://www.w3.org/2000/svg" className="icon icon-tabler icon-tabler-zoom-in-area w-5 h-5" viewBox="0 0 24 24" strokeWidth="1.5" stroke="#9e9e9e" fill="none" strokeLinecap="round" strokeLinejoin="round">
                                            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                                            <path d="M15 13v4" />
                                            <path d="M13 15h4" />
                                            <path d="M15 15m-5 0a5 5 0 1 0 10 0a5 5 0 1 0 -10 0" />
                                            <path d="M22 22l-3 -3" />
                                            <path d="M6 18h-1a2 2 0 0 1 -2 -2v-1" />
                                            <path d="M3 11v-1" />
                                            <path d="M3 6v-1a2 2 0 0 1 2 -2h1" />
                                            <path d="M10 3h1" />
                                            <path d="M15 3h1a2 2 0 0 1 2 2v1" />
                                        </svg>
                                    </button>
                                {/* </div> */}
                                {/* // <div className="flex"> */}

                                    <button
                                        onClick={() => { mapLayers.deleteLayer(index) }}
                                        className="btn btn-ghost btn-xs">
                                        <svg xmlns="http://www.w3.org/2000/svg" className="icon icon-tabler icon-tabler-x w-5 h-5" viewBox="0 0 24 24" strokeWidth="1.5" stroke="#9e9e9e" fill="none" strokeLinecap="round" strokeLinejoin="round">

                                            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                                            <path d="M18 6l-12 12" />
                                            <path d="M6 6l12 12" />
                                        </svg>
                                    </button>
                                {/* </div> */}
                            </div>
                        );
                    })
                }
            </div>
        ),
        [mapLayers.data.layers],
    )

    return (
        <div className="leaflet-bottom leaflet-right">
            <div className="leaflet-control">{minimap}</div>
        </div>
    )
}
