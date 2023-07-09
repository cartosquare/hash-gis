import L, { Bounds, LatLngExpression } from 'leaflet';

export const createLeafletBounds = (extent: number[]): L.LatLngBounds => {
    return new L.LatLngBounds(L.latLng(extent[0], extent[1]), L.latLng(extent[2], extent[3]));
}