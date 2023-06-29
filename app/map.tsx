import 'leaflet/dist/leaflet.css';
import { MapContainer, TileLayer, Marker, useMap, FeatureGroup, Popup } from 'react-leaflet';
import L, { LatLngExpression } from 'leaflet';

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


export default function Map({
  layers,
  bounds,
}: {
  layers: string[],
  bounds: L.LatLngBounds | undefined,
}) {
  return (
    <MapContainer className='flex grow' bounds={bounds}>
      {
        layers && layers.map((url, index) => (
          <TileLayer
            attribution='&copy; <a href="https://www.rs.sensetime.com/">SenseTime</a>'
            url={url}
          />
        ))
      }
      <BoundsView bounds={bounds} />
    </MapContainer>
  );
}

