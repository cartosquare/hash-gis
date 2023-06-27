import 'leaflet/dist/leaflet.css';
import { MapContainer, TileLayer, Marker, useMap, FeatureGroup, Popup } from 'react-leaflet';
import { useRef, useState } from 'react';
import L, { LatLngExpression } from 'leaflet';
// import 'leaflet-draw/dist/leaflet.draw.css';
// import { EditControl } from 'react-leaflet-draw';

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

export default function Map(){
 const [geoData, setGeoData] = useState({ lat: 27.67993181471809, lng: 118.3639870939991 });
  const center: LatLngExpression = [geoData.lat, geoData.lng];
  return (
    <MapContainer className='flex grow' center={center} zoom={12}>
      <TileLayer
        attribution='&copy; <a href="https://www.mapbox.com/about/maps/">Mapbox</a> &copy; <a href="http://osm.org/copyright">OpenStreetMap</a> &copy; <a href="https://www.maxar.com/">Maxar</a>'
        url='https://api.mapbox.com/v4/mapbox.satellite/{z}/{x}/{y}.webp?sku=101nCfuWrTLLf&access_token=pk.eyJ1IjoiYXZvbG92aWsiLCJhIjoiY2txdzNpdWs1MGkwZjJ3cGNrYnZua3I4aCJ9.Le6NapjFYy5FfdDXfBmvrg'
      />
      <ChangeView coords={center} />
      <Marker position={center}>
      <Popup>
        A pretty CSS3 popup. <br /> Easily customizable.
      </Popup>
    </Marker>

    </MapContainer>
  );
}

