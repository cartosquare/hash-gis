'use client';

import 'leaflet/dist/leaflet.css';
import { MapContainer, TileLayer, Marker, useMap, FeatureGroup, Popup, LayersControl } from 'react-leaflet';
import L, { Bounds, LatLngExpression } from 'leaflet';
import { useEffect, useState } from 'react';
import { MapLegend } from './components/map-legend';
import { useMapLayers } from './context/maplayers-context';
import { createLeafletBounds } from './utils';

L.Icon.Default.mergeOptions({
  iconRetinaUrl: ('leaflet/images/marker-icon-2x.png'),
  iconUrl: ('leaflet/images/marker-icon.png'),
  shadowUrl: ('leaflet/images/marker-shadow.png')
});

export function ChangeView({ coords }: { coords: LatLngExpression }) {
  const map = useMap();
  map.setView(coords, 12);
  return null;
}

export function BoundsView({ bounds }: { bounds: L.LatLngBounds | undefined }) {
  const map = useMap();

  useEffect(() => {
    console.log("into bounds view")
    if (bounds) {
      map.fitBounds(bounds)
    }
  }, [bounds])
  return null;
}

export default function MapSquare() {
  const [bounds, setBounds] = useState<L.LatLngBounds>();
  const mapLayers = useMapLayers();

  useEffect(() => {
    console.log("layer changed: ", mapLayers.layers);
    if (mapLayers.layers.length == 0) {
      return;
    }

    let b: L.LatLngBounds = createLeafletBounds(mapLayers.layers[0].bounds as number[]);
    for (let index = 1; index < mapLayers.layers.length; index++) {
      b.extend(createLeafletBounds(mapLayers.layers[index].bounds as number[]));
    }
    setBounds(b);
  }, [mapLayers.layers])

  return (
    <MapContainer className='flex grow' center={L.latLng(39.98, 116.31)} zoom={10}>
      {
        mapLayers.layers.map((s, index) => s.show && (
          <TileLayer key={index}
            url={`http://localhost:8080/${s.name}/{z}/{x}/{y}.png`}
          />
        ))
      }
      {
        bounds && <BoundsView bounds={bounds} />
      }
      <MapLegend></MapLegend>
    </MapContainer>
  );
}

