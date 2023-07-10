"use client"

import { createContext, PropsWithChildren, useContext, useState } from "react";
import { MapSettings } from "../types";
import { Body, fetch } from '@tauri-apps/api/http';
import { message } from "@tauri-apps/api/dialog";

type mapLayersType = {
    layers: MapSettings[],
    createLayer: (filepath: string, geoType: string) => void;
    deleteLayer: (index: number) => void;
    clearLayers: () => void;
    toggleLayer: (index: number) => void;
    randomLayerColor: (index: number) => void;
}


const mapLayersContextDefaultValue: mapLayersType = {
    layers: [],
    createLayer: (filepath: string, geoType: string) => { },
    deleteLayer: (index: number) => { },
    clearLayers: () => { },
    toggleLayer: (index: number) => { },
    randomLayerColor: (index: number) => { },
};

// create context
const MapLayersContext = createContext<mapLayersType>(mapLayersContextDefaultValue);

// create context
export const useMapLayers = (): mapLayersType => {
    return useContext(MapLayersContext);
}

// create provider
export const MapLayersProvider = (props: PropsWithChildren) => {
    const [mapLayers, setMapLayers] = useState<MapSettings[]>([]);

    const clearLayers = () => {
        setMapLayers([]);
    }

    const randomLayerColor = async (index: number) => {
        if (index >= 0 && index < mapLayers.length) {
            let path = mapLayers[index].path;
            let layer_type = mapLayers[index].xml ? 'vector' : 'raster';
            // create new layer
            const newMapLayer = await createMapLayer(path, layer_type);
            if (newMapLayer) {
                newMapLayer.show = true;

                let newLayers = [];
                for (let i = 0; i < mapLayers.length; i++) {
                    if (i == index) {
                        newLayers.push(newMapLayer);
                    } else {
                        newLayers.push(mapLayers[i])
                    }
                }
                setMapLayers(newLayers);
            }
        }
    }

    const toggleLayer = (index: number) => {
        if (index >= 0 && index < mapLayers.length) {
            let newLayers = [];
            for (let i = 0; i < mapLayers.length; i++) {
                if (i == index) {
                    mapLayers[i].show = !mapLayers[i].show;
                }
                newLayers.push(mapLayers[i])
            }
            setMapLayers(newLayers);
        }
    }

    const deleteLayer = (index: number) => {
        if (index >= 0 && index < mapLayers.length) {
            let newLayers = [];
            for (let i = 0; i < mapLayers.length; i++) {
                if (i != index) {
                    newLayers.push(mapLayers[i])
                }
            }
            setMapLayers(newLayers);
        }
    }

    const createLayer = async (filepath: string, geoType: string) => {
        const newMaplayer = await createMapLayer(filepath, geoType);
        if (newMaplayer) {
            newMaplayer.show = true;
            setMapLayers([...mapLayers, newMaplayer]);
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
            show: true,
        };

        const colors = [
            '#8e0152',
            '#c51b7d',
            '#de77ae',
            '#f1b6da',
            '#fde0ef',
            '#e6f5d0',
            '#b8e186',
            '#7fbc41',
            '#4d9221',
            '#276419'];

        if (geoType == "vector") {
            const color = colors[Math.floor(Math.random() * colors.length)];

            console.log('color', color);
            const style = `
<Map srs="epsg:3857">
	<Style name="My Style">
		<Rule>
			<PolygonSymbolizer fill="${color}" fill-opacity="0.5"/>
			<LineSymbolizer stroke="${color}" stroke-opacity="1" stroke-width="1"/>
		</Rule>
	</Style>
	<Layer name="" srs="epsg:4326">
		<StyleName>My Style</StyleName>
		<Datasource>
			<Parameter name="file">${filepath}</Parameter>
			<Parameter name="layer_by_index">0</Parameter>
			<Parameter name="type">ogr</Parameter>
		</Datasource>
	</Layer>
</Map>
    `
            bodyData.xml = style;

        }

        try {
            const rawResponse = await fetch<MapSettings>(`http://localhost:8080/map`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: Body.json(bodyData),
            });
            if (rawResponse.status != 200) {
                console.log('error', rawResponse);
                message("加载图层失败！请检查数据有效性。", {title: "加载图层失败", type: "error"})
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
            message("加载图层失败！请检查数据有效性。", {title: "加载图层失败", type: "error"})
        }
    }


    return (
        <MapLayersContext.Provider
            value={{
                layers: mapLayers,
                createLayer,
                deleteLayer,
                clearLayers,
                toggleLayer,
                randomLayerColor,
            }}
        >
            {props.children}
        </MapLayersContext.Provider>
    );
}

