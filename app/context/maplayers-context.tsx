"use client"

import { createContext, PropsWithChildren, useContext, useState } from "react";
import { MapSettings } from "../types";
import { Body, fetch } from '@tauri-apps/api/http';
import { message } from "@tauri-apps/api/dialog";

type mapLayersStatus = {
    layers: MapSettings[],
    visible: boolean[],
    initialized: boolean,
}

const defaultMapLayerStatus = (): mapLayersStatus => {
    return {
        layers: [],
        visible: [],
        initialized: false,
    };
}

type mapLayersType = {
    data: mapLayersStatus,
    createLayer: (filepath: string, geoType: string) => void;
    deleteLayer: (index: number) => void;
    clearLayers: () => void;
    toggleLayer: (index: number) => void;
    refreshLayer: (index: number) => void;
}


const mapLayersContextDefaultValue: mapLayersType = {
    data: defaultMapLayerStatus(),
    createLayer: (filepath: string, geoType: string) => { },
    deleteLayer: (index: number) => { },
    clearLayers: () => { },
    toggleLayer: (index: number) => { },
    refreshLayer: (index: number) => { },
};

// create context
const MapLayersContext = createContext<mapLayersType>(mapLayersContextDefaultValue);

// create context
export const useMapLayers = (): mapLayersType => {
    return useContext(MapLayersContext);
}

// create provider
export const MapLayersProvider = (props: PropsWithChildren) => {
    const [mapLayers, setMapLayers] = useState<mapLayersStatus>(defaultMapLayerStatus());

    const clearLayers = () => {
        setMapLayers(defaultMapLayerStatus());
    }

    const refreshLayer = async (index: number) => {
        if (index >= 0 && index < mapLayers.layers.length) {
            let path = mapLayers.layers[index].path;
            let layer_type = mapLayers.layers[index].geo_type
            // create new layer
            const newMapLayer = await createMapLayer(path, layer_type);
            if (newMapLayer) {
                mapLayers.visible[index] = true;

                let newLayers = [];
                for (let i = 0; i < mapLayers.layers.length; i++) {
                    if (i == index) {
                        newLayers.push(newMapLayer);
                    } else {
                        newLayers.push(mapLayers.layers[i])
                    }
                }
                setMapLayers({
                    ...mapLayers,
                    layers: newLayers,
                });
            }
        }
    }

    const toggleLayer = (index: number) => {
        if (index >= 0 && index < mapLayers.layers.length) {
            for (let i = 0; i < mapLayers.layers.length; i++) {
                if (i == index) {
                    mapLayers.visible[i] = !mapLayers.visible[i];
                }
            }
            setMapLayers({
                ...mapLayers
            });
        }
    }

    const deleteLayer = (index: number) => {
        if (index >= 0 && index < mapLayers.layers.length) {
            let newLayers = [];
            for (let i = 0; i < mapLayers.layers.length; i++) {
                if (i != index) {
                    newLayers.push(mapLayers.layers[i])
                }
            }
            setMapLayers({
                ...mapLayers,
                initialized: newLayers.length > 0,
                layers: newLayers,
            });
        }
    }

    const createLayer = async (filepath: string, geoType: string) => {
        const newMaplayer = await createMapLayer(filepath, geoType);
        if (newMaplayer) {
            if (newMaplayer.extent
                && (newMaplayer.extent.width > 20000 || newMaplayer.extent.height > 20000)
                && !newMaplayer.has_overview) {
                message("为了获得更流畅的浏览体验，请先对数据建立金字塔。", { title: "数据没有金字塔", type: "info" });
            }
            setMapLayers({
                ...mapLayers,
                initialized: true,
                layers: [...mapLayers.layers, newMaplayer],
                visible: [...mapLayers.visible, true],
            });
        }
    }

    const createMapLayer = async (filepath: string, geoType: string) => {
        let bodyData: MapSettings = {
            name: "",
            path: filepath,
            xml: null,
            extent: null,
            geotransform: null,
            style: null,
            no_data_value: null,
            spatial_info: null,
            spatial_units: null,
            driver_name: null,
            bounds: null,
            has_overview: false,
            geo_type: geoType,
        };
        try {
            const rawResponse = await fetch<MapSettings>(`http://localhost:28904/map`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: Body.json(bodyData),
            });
            if (rawResponse.status != 200) {
                console.log('error', rawResponse);
                message("加载图层失败！请检查数据有效性。", { title: "加载图层失败", type: "error" })
            } else {
                const response = rawResponse.data;
                if (response.bounds) {
                    // console.log(response);

                    return response;
                }
            }
        } catch (error) {
            console.log(error);
            // toast.error(`请求失败：${error}`);
            console.log('error!', error);
            message("加载图层失败！请检查数据有效性。", { title: "加载图层失败", type: "error" })
        }
    }


    return (
        <MapLayersContext.Provider
            value={{
                data: mapLayers,
                createLayer,
                deleteLayer,
                clearLayers,
                toggleLayer,
                refreshLayer,
            }}
        >
            {props.children}
        </MapLayersContext.Provider>
    );
}

