import L, { Bounds, LatLngExpression } from 'leaflet';

export const createLeafletBounds = (extent: number[]): L.LatLngBounds => {
    return new L.LatLngBounds(L.latLng(extent[0], extent[1]), L.latLng(extent[2], extent[3]));
}

export const getPredictMsg = (msg: string): string => {
    if (msg == "loading-model") {
        return "加载模型";
    } else if (msg == "predicting") {
        return "解译";
    } else if (msg == "postprocessing") {
        return "后处理";
    } else {
        return "后处理";
    }
}