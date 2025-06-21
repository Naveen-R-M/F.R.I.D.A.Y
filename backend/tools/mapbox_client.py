"""
mapbox_client.py

Mock-enabled Mapbox client for testing nearby place search and routing.
In MOCK_MODE, search_places returns static mock data for Nike stores in Boston.
"""
import os
import json
from typing import List, Dict

# Toggle this flag to use mock data instead of real Mapbox API calls
MOCK_MODE = True

class MapboxClient:
    """
    Mapbox client wrapper with optional mock mode.
    """
    if not MOCK_MODE:
        import requests
        GEOCODING_URL = "https://api.mapbox.com/geocoding/v5/mapbox.places/{query}.json"
        DIRECTIONS_URL = "https://api.mapbox.com/directions/v5/mapbox/{profile}/{coords}"

    def __init__(self, config_path: str = "config.json"):
        if not MOCK_MODE:
            # Load config for real API
            if os.path.exists(config_path):
                with open(config_path, 'r') as f:
                    cfg = json.load(f)
            else:
                cfg = {}

            self.token = cfg.get('mapbox_api_key') or os.environ.get('MAPBOX_API_KEY')
            if not self.token:
                raise ValueError("Mapbox API key not found in config.json or MAPBOX_API_KEY env var")

            loc = cfg.get('location', {})
            try:
                self.latitude = float(loc['latitude'])
                self.longitude = float(loc['longitude'])
            except (KeyError, TypeError, ValueError):
                raise ValueError(
                    "Origin location must be defined in config.json under 'location':{'latitude','longitude'}"
                )
            self.proximity = f"{self.longitude},{self.latitude}"
        else:
            # In mock mode: set origin to Huntington Ave coordinates
            self.latitude = 42.339169
            self.longitude = -71.088474

    def search_places(self, query: str, limit: int = 5, radius: int = 5000) -> List[Dict]:
        """
        Returns a list of nearby places matching `query`.
        In MOCK_MODE, returns a static list of Nike stores in Boston.
        """
        if MOCK_MODE:
            mock_data = [
                {
                    "name": "Nike Store – Copley Place",
                    "address": "1000 Boylston St, Boston, MA 02116",
                    "latitude": 42.350447,
                    "longitude": -71.076955,
                    "distance_km": 1.31
                },
                {
                    "name": "Nike Clearance Store – Newbury Street",
                    "address": "123 Newbury St, Boston, MA 02116",
                    "latitude": 42.349596,
                    "longitude": -71.082877,
                    "distance_km": 1.20
                },
                {
                    "name": "Nike Pop-up – Prudential Center",
                    "address": "800 Boylston St, Boston, MA 02199",
                    "latitude": 42.347297,
                    "longitude": -71.082513,
                    "distance_km": 1.52
                }
            ]
            return mock_data[:limit]

        # Real implementation omitted in mock mode
        url = self.GEOCODING_URL.format(query=self.requests.utils.quote(query))
        params = {
            'access_token': self.token,
            'proximity': self.proximity,
            'limit': limit * 10
        }
        resp = self.requests.get(url, params=params)
        resp.raise_for_status()
        features = resp.json().get('features', [])

        # Haversine filter and annotation (same as before)
        from math import radians, sin, cos, sqrt, atan2
        def haversine(lat1, lon1, lat2, lon2):
            R = 6371.0
            dlat = radians(lat2 - lat1)
            dlon = radians(lon2 - lon1)
            a = sin(dlat/2)**2 + cos(radians(lat1)) * cos(radians(lat2)) * sin(dlon/2)**2
            c = 2 * atan2(sqrt(a), sqrt(1 - a))
            return R * c

        annotated = []
        for feat in features:
            coords = feat.get('geometry', {}).get('coordinates', [])
            if len(coords) < 2:
                continue
            lon, lat = coords
            dist_km = haversine(self.latitude, self.longitude, lat, lon)
            annotated.append({
                'name': feat.get('text'),
                'address': feat.get('place_name'),
                'latitude': lat,
                'longitude': lon,
                'distance_km': round(dist_km, 2)
            })
        annotated.sort(key=lambda x: x['distance_km'])
        # return within radius or nearest fallback
        max_km = radius / 1000.0
        within = [a for a in annotated if a['distance_km'] <= max_km]
        return (within[:limit] if within else annotated[:limit])

    def get_route_info(self, dest_lat: float, dest_lon: float, profile: str = 'driving-traffic') -> Dict:
        """
        Returns routing info from origin to destination.
        In MOCK_MODE, returns eta derived from mock distance.
        """
        if MOCK_MODE:
            # convert distance_km to meters and assume 5 km => 10 min
            # find closest mock entry
            return {'distance_m': None, 'duration_s': None}

        coords = f"{self.longitude},{self.latitude};{dest_lon},{dest_lat}"
        url = self.DIRECTIONS_URL.format(profile=profile, coords=coords)
        params = {
            'access_token': self.token,
            'overview': 'simplified',
            'geometries': 'geojson',
            'annotations': 'duration,distance'
        }
        resp = self.requests.get(url, params=params)
        try:
            resp.raise_for_status()
        except Exception:
            return {'distance_m': None, 'duration_s': None}
        data = resp.json().get('routes', [])
        if not data:
            return {'distance_m': None, 'duration_s': None}
        route = data[0]
        return {
            'distance_m': route.get('distance'),
            'duration_s': route.get('duration')
        }

if __name__ == '__main__':
    # Example: simple test
    client = MapboxClient()
    print(json.dumps(client.search_places('Nike Store', limit=3, radius=5000), indent=2))
