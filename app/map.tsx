'use client';

import 'leaflet/dist/leaflet.css';
import { MapContainer, TileLayer, Marker, useMap, FeatureGroup, Popup, LayersControl } from 'react-leaflet';
import L, { Bounds, LatLngExpression } from 'leaflet';
import { MapSettings } from './types';
import { useEffect, useState } from 'react';

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
  if (bounds) {
    map.fitBounds(bounds)
  }
  return null;
}


export default function MapSquare({
  settings,
}: {
  settings: MapSettings[],
}) {
  const [bounds, setBounds] = useState<L.LatLngBounds>();

  const createLeafletBounds = (extent: number[]): L.LatLngBounds => {
      return new L.LatLngBounds(L.latLng(extent[0], extent[1]), L.latLng(extent[2], extent[3]));
  }

  useEffect(() => {
    if (settings.length == 0) {
      return;
    }

    console.log(settings);

    let b: L.LatLngBounds = createLeafletBounds(settings[0].bounds as number[]);
    for (let index = 1; index < settings.length; index++) {
      b.extend(createLeafletBounds(settings[index].bounds as number[]));
    }
    console.log('bounds: ', b);
    setBounds(b);
  }, [settings])
  return (
    <MapContainer className='flex grow' bounds={bounds}>

      <LayersControl position='topright'>

        {
          settings && settings.map((s, index) => (

            <LayersControl.Overlay key={index} name={`${s.path.replace(/^.*[\\\/]/, '')}`} checked>
              <TileLayer
                attribution='&copy; <a href="https://www.rs.sensetime.com/">SenseTime</a>'
                url={`http://localhost:8080/${s.name}/{z}/{x}/{y}.png`}
              />
            </LayersControl.Overlay>
          ))
        }
      </LayersControl>
      <BoundsView bounds={bounds} />
    </MapContainer>
  );
}

